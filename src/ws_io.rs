use crate::{ import::*, WsErr, WsErrKind, WsMessage, WsState, WsEvent, notify };


/// A futures 0.3 Sink/Stream of [WsMessage]. It further implements AsyncRead/AsyncWrite
/// that can be framed with codecs. You can use the compat layer from the futures library if you want to
/// use tokio codecs. See the [integration tests](https://github.com/ws_stream_wasm/tree/master/tests/tokio_codec.rs)
/// if you need an example.
///
/// Created with [WsStream::connect](crate::WsStream::connect).
//
pub struct WsIo
{
	ws: Rc< WebSocket >,

	// The queue of received messages
	//
	queue: Rc<RefCell< VecDeque<WsMessage> >>,

	// Last waker of task that wants to read incoming messages to be woken up on a new message
	//
	waker: Rc<RefCell< Option<Waker> >>,

	// Last waker of task that wants to write to the Sink
	//
	sink_waker: Rc<RefCell< Option<Waker> >>,

	// A pointer to the pharos of WsStream for when we need to listen to events
	//
	pharos: Rc<RefCell< Pharos<WsEvent> >>,

	// State information for partially read messages in AsyncRead
	//
	state: ReadState,

	// The closure that will receive the messages
	//
	_on_mesg: Closure< dyn FnMut( MessageEvent ) >,

	// This allows us to store a future to poll when Sink::poll_close is called
	//
	closer: Option< Events<WsEvent> >,
}

unsafe impl Send for WsIo {}
unsafe impl Sync for WsIo {}

impl WsIo
{
	/// Create a new WsIo.
	//
	pub fn new( ws: Rc<WebSocket>, pharos : Rc<RefCell< Pharos<WsEvent> >> ) -> Self
	{
		let waker     : Rc<RefCell<Option<Waker>>> = Rc::new( RefCell::new( None ));
		let sink_waker: Rc<RefCell<Option<Waker>>> = Rc::new( RefCell::new( None ));

		let state = ReadState::PendingChunk;
		let queue = Rc::new( RefCell::new( VecDeque::new() ) );
		let q2    = queue.clone();
		let w2    = waker.clone();


		// Send the incoming ws messages to the WsStream object
		//
		#[ allow( trivial_casts ) ]
		//
		let on_mesg = Closure::wrap( Box::new( move |msg_evt: MessageEvent|
		{
			trace!( "WsStream: message received!" );

			q2.borrow_mut().push_back( WsMessage::from( msg_evt ) );

			if let Some( w ) = w2.borrow_mut().take()
			{
				trace!( "WsStream: waking up task" );
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
			ws                ,
			queue             ,
			state             ,
			waker             ,
			sink_waker        ,
			pharos            ,
			closer  : None    ,
			_on_mesg: on_mesg ,
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
}



impl fmt::Debug for WsIo
{
	fn fmt( &self, f: &mut fmt::Formatter<'_> ) -> fmt::Result
	{
		write!( f, "WsIo for connection: {}", self.ws.url() )
	}
}



impl Drop for WsIo
{
	// We don't block here, just tell the browser to close the connection and move on.
	//
	fn drop( &mut self )
	{
		trace!( "Drop WsIo" );

		match self.ready_state()
		{
			WsState::Closing | WsState::Closed => {}

			_ =>
			{
				// This can't fail
				//
				self.ws.close_with_code( 1000 ).expect( "WsIo::drop - close ws socket" );


				// Notify Observers
				//
				notify( self.pharos.clone(), WsEvent::Closing )
			}
		}

		self.ws.set_onmessage( None );
	}
}



impl Stream for WsIo
{
	type Item = WsMessage;

	// Currently requires an unfortunate copy from Js memory to WASM memory. Hopefully one
	// day we will be able to receive the MessageEvt directly in WASM.
	//
	fn poll_next( mut self: Pin<&mut Self>, cx: &mut Context<'_> ) -> Poll<Option< Self::Item >>
	{
		trace!( "WsIo as Stream gets polled" );

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



impl Sink<WsMessage> for WsIo
{
	type Error = WsErr;


	// Web API does not really seem to let us check for readiness, other than the connection state.
	//
	fn poll_ready( mut self: Pin<&mut Self>, cx: &mut Context<'_> ) -> Poll<Result<(), Self::Error>>
	{
		trace!( "Sink<WsMessage> for WsIo: poll_ready" );

		match self.ready_state()
		{
			WsState::Connecting =>
			{
				*self.sink_waker.borrow_mut() = Some( cx.waker().clone() );

				Poll::Pending
			}

			WsState::Open => Ok(()).into(),
			_             => Err( WsErrKind::ConnectionNotOpen.into() ).into(),
		}
	}


	fn start_send( self: Pin<&mut Self>, item: WsMessage ) -> Result<(), Self::Error>
	{
		trace!( "Sink<WsMessage> for WsIo: start_send, state is {:?}", self.ready_state() );

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
					WsMessage::Binary( mut d ) => { trace!( "binary message: {:?}", d ); self.ws.send_with_u8_array( &mut d ).map_err( |_| WsErrKind::ConnectionNotOpen)?; }
					WsMessage::Text  (     s ) => { trace!( "text message" ); self.ws.send_with_str     ( &    s ).map_err( |_| WsErrKind::ConnectionNotOpen)?; }
				}

				Ok(())
			},


			// Connecting, Closing or Closed
			//
			_ => Err( WsErrKind::ConnectionNotOpen.into() ),
		}
	}



	fn poll_flush( self: Pin<&mut Self>, _: &mut Context<'_> ) -> Poll<Result<(), Self::Error>>
	{
		trace!( "Sink<WsMessage> for WsIo: poll_flush" );

		Ok(()).into()
	}



	// TODO: find a simpler implementation, notably this needs to spawn a future.
	//       this can be done by creating a custom future. If we are going to implement
	//       events with pharos, that's probably a good time to re-evaluate this.
	//
	fn poll_close( mut self: Pin<&mut Self>, cx: &mut Context<'_> ) -> Poll<Result<(), Self::Error>>
	{
		trace!( "Sink<WsMessage> for WsIo: poll_close" );

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



impl AsyncWrite for WsIo
{
	fn poll_write( mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8] ) -> Poll<Result<usize, io::Error>>
	{
		let res = ready!( self.as_mut().poll_ready( cx ) );

		match res
		{
			Ok(_) =>
			{
				let n = buf.len();

				match self.start_send( WsMessage::Binary( buf.into() ) )
				{
					Ok (_) =>  return Ok(n).into(),

					Err(e) if e.kind() == &WsErrKind::ConnectionNotOpen =>

						return Poll::Ready( Err( io::Error::from( io::ErrorKind::NotConnected ))) ,

					// This shouldn't happen, so panic for early detection.
					//
					Err(_) => unreachable!(),
				}
			}

			Err(e) if e.kind() == &WsErrKind::ConnectionNotOpen =>

				return Poll::Ready( Err( io::Error::from( io::ErrorKind::NotConnected ))),

			Err(_) => unreachable!(),
		}
	}



	fn poll_flush( self: Pin<&mut Self>, _cx: &mut Context<'_> ) -> Poll<Result<(), io::Error>>
	{
		Poll::Ready( Ok(()) )
	}


	fn poll_close( self: Pin<&mut Self>, cx: &mut Context<'_> ) -> Poll<Result<(), io::Error>>
	{
		let _ = ready!( < Self as Sink<WsMessage> >::poll_close( self, cx ) );

		// WsIo poll_close is infallible
		//
		Ok(()).into()
	}
}






#[derive(Debug, Clone)]
//
enum ReadState
{
	Ready { chunk: Vec<u8>, chunk_start: usize },
	PendingChunk,
}



impl AsyncRead for WsIo
{
	fn poll_read( mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut [u8] ) -> Poll< Result<usize, io::Error> >
	{
		trace!( "WsIo - AsyncRead: poll_read called" );

		loop
		{
			match &mut self.state
			{
				ReadState::Ready { chunk, chunk_start } =>
				{
					let end = cmp::min( *chunk_start + buf.len(), chunk.len() );
					let len = end - *chunk_start;

					buf[..len].copy_from_slice( &chunk[ *chunk_start..end ] );


					if chunk.len() == end { self.state = ReadState::PendingChunk }
					else                  { *chunk_start = end                   }

					return Ok(len).into();
				}


				ReadState::PendingChunk =>
				{
					trace!( "poll_read: pending" );

					match Pin::new( &mut self ).poll_next(cx)
					{
						// We have a message
						//
						Poll::Ready( Some(chunk) ) =>
						{
							self.state = ReadState::Ready { chunk: chunk.into(), chunk_start: 0 };
							continue;
						}

						// The stream has ended
						//
						Poll::Ready( None ) =>
						{
							trace!( "poll_read: stream has ended" );
							return Ok(0).into();
						}

						// No chunk yet, save the task to be woken up
						//
						Poll::Pending =>
						{
							trace!( "poll_read: return Pending" );
							return Poll::Pending;
						}
					}
				}
			}
		}
	}
}




#[ cfg( feature = "tokio_io" ) ]
//
#[ cfg_attr( feature = "docs", doc(cfg( feature = "tokio_io" )) ) ]
//
impl TokAsyncWrite for WsIo
{
	fn poll_write( self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8] ) -> Poll<Result<usize, io::Error>>
	{
		AsyncWrite::poll_write( self, cx, buf )
	}



	fn poll_flush( self: Pin<&mut Self>, cx: &mut Context<'_> ) -> Poll<Result<(), io::Error>>
	{
		AsyncWrite::poll_flush( self, cx )
	}


	fn poll_shutdown( self: Pin<&mut Self>, cx: &mut Context<'_> ) -> Poll<Result<(), io::Error>>
	{
		AsyncWrite::poll_close( self, cx )
	}
}


#[ cfg( feature = "tokio_io" ) ]
//
#[ cfg_attr( feature = "docs", doc(cfg( feature = "tokio_io" )) ) ]
//
impl TokAsyncRead for WsIo
{
	fn poll_read( self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut [u8] ) -> Poll< Result<usize, io::Error> >
	{
		AsyncRead::poll_read( self, cx, buf )
	}
}



