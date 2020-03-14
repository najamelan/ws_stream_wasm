use crate::{ import::*, * };


/// A futures 0.3 Sink/Stream of [WsMessage]. It further implements AsyncRead/AsyncWrite
/// that can be framed with codecs. You can use the compat layer from the futures library if you want to
/// use tokio codecs. See the [integration tests](https://github.com/ws_stream_wasm/tree/master/tests/tokio_codec.rs)
/// if you need an example.
///
/// Created with [WsMeta::connect](crate::WsMeta::connect).
//
pub struct WsStream
{
	ws: SendWrapper< Rc< WebSocket > >,

	// The queue of received messages
	//
	queue: SendWrapper< Rc<RefCell< VecDeque<WsMessage> >> >,

	// Last waker of task that wants to read incoming messages to be woken up on a new message
	//
	waker: SendWrapper< Rc<RefCell< Option<Waker> >> >,

	// Last waker of task that wants to write to the Sink
	//
	sink_waker: SendWrapper< Rc<RefCell< Option<Waker> >> >,

	// A pointer to the pharos of WsMeta for when we need to listen to events
	//
	pharos: SendWrapper< Rc<RefCell< Pharos<WsEvent> >> >,

	// The closure that will receive the messages
	//
	_on_mesg: SendWrapper< Closure< dyn FnMut( MessageEvent ) > >,

	// This allows us to store a future to poll when Sink::poll_close is called
	//
	closer: Option< Events<WsEvent> >,
}


impl WsStream
{
	/// Create a new WsStream.
	//
	pub(crate) fn new( ws: SendWrapper< Rc<WebSocket> >, pharos : SendWrapper< Rc<RefCell< Pharos<WsEvent> >> > ) -> Self
	{
		let waker     : SendWrapper< Rc<RefCell<Option<Waker>>> > = SendWrapper::new( Rc::new( RefCell::new( None )) );
		let sink_waker: SendWrapper< Rc<RefCell<Option<Waker>>> > = SendWrapper::new( Rc::new( RefCell::new( None )) );

		let queue = SendWrapper::new( Rc::new( RefCell::new( VecDeque::new() ) ) );
		let q2    = queue.clone();
		let w2    = waker.clone();


		// Send the incoming ws messages to the WsMeta object
		//
		#[ allow( trivial_casts ) ]
		//
		let on_mesg = Closure::wrap( Box::new( move |msg_evt: MessageEvent|
		{
			trace!( "WsMeta: message received!" );

			q2.borrow_mut().push_back( WsMessage::from( msg_evt ) );

			if let Some( w ) = w2.borrow_mut().take()
			{
				trace!( "WsMeta: waking up task" );
				w.wake()
			}

		}) as Box< dyn FnMut( MessageEvent ) > );


		// Install callback
		//
		ws.set_onmessage  ( Some( on_mesg.as_ref().unchecked_ref() ) );


		// When the connection closes, we need to verify if there are any tasks
		// waiting on poll_next. We need to wake them up.
		//
		let ph    = pharos.clone();
		let wake  = waker.clone();
		let swake = sink_waker.clone();

		let wake_on_close = async move
		{
			let mut rx;

			// Scope to avoid borrowing across await point.
			//
			{
				match ph.borrow_mut().observe( Filter::Pointer( WsEvent::is_closed ).into() )
				{
					Ok(events) => rx = events               ,
					Err(e)     => unreachable!( "{:?}", e ) , // only happens if we closed it.
				}
			}

			rx.next().await;

			if let Some(w) = &*wake.borrow()
			{
				w.wake_by_ref();
			}

			if let Some(w) = &*swake.borrow()
			{
				w.wake_by_ref();
			}
		};

		spawn_local( wake_on_close );


		Self
		{
			ws                                    ,
			queue                                 ,
			waker                                 ,
			sink_waker                            ,
			pharos                                ,
			closer  : None                        ,
			_on_mesg: SendWrapper::new( on_mesg ) ,
		}
	}



	/// Verify the [WsState] of the connection.
	//
	pub fn ready_state( &self ) -> WsState
	{
		self.ws.ready_state().try_into().map_err( |e| error!( "{}", e ) )

			// This can't throw unless the browser gives us an invalid ready state
			//
			.expect_throw( "Convert ready state from browser API" )
	}



	/// Access the wrapped [web_sys::WebSocket](https://docs.rs/web-sys/0.3.25/web_sys/struct.WebSocket.html) directly.
	///
	/// `ws_stream_wasm` tries to expose all useful functionality through an idiomatic rust API, so hopefully
	/// you won't need this, however if I missed something, you can.
	///
	/// ## Caveats
	/// If you call `set_onopen`, `set_onerror`, `set_onmessage` or `set_onclose` on this, you will overwrite
	/// the event listeners from `ws_stream_wasm`, and things will break.
	//
	pub fn wrapped( &self ) -> &WebSocket
	{
		&self.ws
	}


	/// Wrap this object in [`IoStream`]. `IoStream` implements AsyncRead/AsyncWrite.
	/// Beware that this will transparenty interprete text messages to bytes.
	//
	pub fn into_io( self ) -> IoStream< WsStreamIo, Vec<u8> >
	{
		IoStream::new( WsStreamIo::new( self ) )
	}
}



impl fmt::Debug for WsStream
{
	fn fmt( &self, f: &mut fmt::Formatter<'_> ) -> fmt::Result
	{
		write!( f, "WsStream for connection: {}", self.ws.url() )
	}
}



impl Drop for WsStream
{
	// We don't block here, just tell the browser to close the connection and move on.
	//
	fn drop( &mut self )
	{
		trace!( "Drop WsStream" );

		match self.ready_state()
		{
			WsState::Closing | WsState::Closed => {}

			_ =>
			{
				// This can't fail
				//
				self.ws.close_with_code( 1000 ).expect( "WsStream::drop - close ws socket" );


				// Notify Observers. This event is not emitted by the websocket API.
				//
				notify( self.pharos.clone(), WsEvent::Closing )
			}
		}

		self.ws.set_onmessage( None );
	}
}



impl Stream for WsStream
{
	type Item = WsMessage;

	// Currently requires an unfortunate copy from Js memory to WASM memory. Hopefully one
	// day we will be able to receive the MessageEvt directly in WASM.
	//
	fn poll_next( mut self: Pin<&mut Self>, cx: &mut Context<'_> ) -> Poll<Option< Self::Item >>
	{
		trace!( "WsStream as Stream gets polled" );

		// Once the queue is empty, check the state of the connection.
		// When it is closing or closed, no more messages will arrive, so
		// return Poll::Ready( None )
		//
		if self.queue.borrow().is_empty()
		{
			*self.waker.borrow_mut() = Some( cx.waker().clone() );

			match self.ready_state()
			{
				WsState::Open | WsState::Connecting => Poll::Pending ,
				_                                   => None.into()   ,
			}
		}

		// As long as there is things in the queue, just keep reading
		//
		else { self.queue.borrow_mut().pop_front().into() }
	}
}



impl Sink<WsMessage> for WsStream
{
	type Error = WsErr;


	// Web API does not really seem to let us check for readiness, other than the connection state.
	//
	fn poll_ready( mut self: Pin<&mut Self>, cx: &mut Context<'_> ) -> Poll<Result<(), Self::Error>>
	{
		trace!( "Sink<WsMessage> for WsStream: poll_ready" );

		match self.ready_state()
		{
			WsState::Connecting =>
			{
				*self.sink_waker.borrow_mut() = Some( cx.waker().clone() );

				Poll::Pending
			}

			WsState::Open => Ok(()).into(),
			_             => Err( WsErr::ConnectionNotOpen.into() ).into(),
		}
	}


	fn start_send( self: Pin<&mut Self>, item: WsMessage ) -> Result<(), Self::Error>
	{
		trace!( "Sink<WsMessage> for WsStream: start_send, state is {:?}", self.ready_state() );

		match self.ready_state()
		{
			WsState::Open =>
			{
				// The send method can return 2 errors:
				// - unpaired surrogates in UTF (we shouldn't get those in rust strings)
				// - connection is already closed.
				//
				// So if this returns an error, we will return ConnectionNotOpen. In principle
				// we just checked that it's open, but this guarantees correctness.
				//
				match item
				{
					WsMessage::Binary( mut d ) => { trace!( "binary message: {:?}", d ); self.ws.send_with_u8_array( &mut d ).map_err( |_| WsErr::ConnectionNotOpen)?; }
					WsMessage::Text  (     s ) => { trace!( "text message" ); self.ws.send_with_str     ( &    s ).map_err( |_| WsErr::ConnectionNotOpen)?; }
				}

				Ok(())
			},


			// Connecting, Closing or Closed
			//
			_ => Err( WsErr::ConnectionNotOpen.into() ),
		}
	}



	fn poll_flush( self: Pin<&mut Self>, _: &mut Context<'_> ) -> Poll<Result<(), Self::Error>>
	{
		trace!( "Sink<WsMessage> for WsStream: poll_flush" );

		Ok(()).into()
	}



	// TODO: find a simpler implementation, notably this needs to spawn a future.
	//       this can be done by creating a custom future. If we are going to implement
	//       events with pharos, that's probably a good time to re-evaluate this.
	//
	fn poll_close( mut self: Pin<&mut Self>, cx: &mut Context<'_> ) -> Poll<Result<(), Self::Error>>
	{
		trace!( "Sink<WsMessage> for WsStream: poll_close" );

		let state = self.ready_state();


		// First close the inner connection
		//
		if state == WsState::Connecting
		|| state == WsState::Open
		{
			// Can't fail
			//
			self.ws.close().unwrap_throw();

			notify( self.pharos.clone(), WsEvent::Closing );
		}


		// Check whether it's closed
		//
		match state
		{
			WsState::Closed =>
			{
				trace!( "WebSocket connection closed!" );
				Ok(()).into()
			}

			_ =>
			{
				// Create a future that will resolve with the close event, so we can
				// poll it.
				//
				if self.closer.is_none()
				{
					let rx = match self.pharos.borrow_mut().observe( Filter::Pointer( WsEvent::is_closed ).into() )
					{
						Ok(events) => events                    ,
						Err(e)     => unreachable!( "{:?}", e ) , // only happens if we closed it.
					};

					self.closer = Some( rx );
				}


				let _ = ready!( Pin::new( &mut self.closer.as_mut().unwrap() ).poll_next(cx) );

				Ok(()).into()
			}
		}
	}
}




