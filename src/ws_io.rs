use
{
	crate :: { import::*, WsErr, WsErrKind, WsMessage, WsState, future_event },
};


/// Sink/Stream of [WsMessage]. Created with [WsStream::connect].
///
#[ allow( dead_code ) ] // we need to store the closure to keep it form being dropped
//
pub struct WsIo
{
	ws     : Rc< WebSocket >                                ,
	on_mesg: Closure< dyn FnMut( MessageEvent ) + 'static > ,
	queue  : Rc<RefCell< VecDeque<WsMessage> >>             ,
	waker  : Rc<RefCell<Option<Waker>>>                     ,
}


impl WsIo
{
	/// Create a new WsIo.
	//
	pub fn new( ws: Rc<WebSocket> ) -> Self
	{
		let waker: Rc<RefCell<Option<Waker>>> = Rc::new( RefCell::new( None ));

		let queue = Rc::new( RefCell::new( VecDeque::new() ) );
		let q2    = queue.clone();
		let w2    = waker.clone();


		// Send the incoming ws messages to the WsStream object
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


		Self
		{
			ws      ,
			queue   ,
			on_mesg ,
			waker   ,
		}
	}



	/// Verify the [WsState] of the connection.
	//
	pub fn ready_state( &self ) -> WsState
	{
		self.ws.ready_state().try_into().map_err( |e| error!( "{}", e ) )

			.expect_throw( "Convert ready state from browser API" )
	}



	/// Access the wrapped [web_sys::WebSocket](https://docs.rs/web-sys/0.3.25/web_sys/struct.WebSocket.html) directly.
	//
	pub fn wrapped( &self ) -> &WebSocket
	{
		&self.ws
	}



	// This method allows to do async close in the poll_close of Sink
	//
	async fn wake_on_close( ws: Rc<WebSocket>, waker: Waker )
	{
		future_event( |cb| ws.set_onclose( cb ) ).await;

		waker.wake();
	}
}



impl fmt::Debug for WsIo
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{
		write!( f, "WsIo for connection: {}", self.ws.url() )
	}
}



impl fmt::Display for WsIo
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{
		write!( f, "WsIo for connection: {}", self.ws.url() )
	}
}



impl Drop for WsIo
{
	// We don't block here, just tell the browser to close the connection and move on.
	// TODO: is this necessary or would it be closed automatically when we drop the WebSocket
	// object? Note that there is also the WsStream which holds a clone.
	//
	fn drop( &mut self )
	{
		trace!( "Drop WsIo" );

		self.ws.close().expect( "WsIo::drop - close ws socket" );
	}
}



impl Stream for WsIo
{
	type Item = WsMessage;

	// Currently requires an unfortunate copy from Js memory to Wasm memory. Hopefully one
	// day we will be able to receive the JsMsgEvent directly in Wasm.
	//
	// TODO: if we would use a channel, this would be simplified somewhat. The question
	//       is what is the performance difference.
	//
	fn poll_next( mut self: Pin<&mut Self>, cx: &mut Context ) -> Poll<Option< Self::Item >>
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
				WsState::Open | WsState::Connecting => Poll::Pending        ,
				_                                   => Poll::Ready  ( None ),
			}
		}

		// As long as there is things in the queue, just keep reading
		//
		else { Poll::Ready( self.queue.borrow_mut().pop_front() ) }
	}
}





impl Sink<WsMessage> for WsIo
{
	type Error = WsErr;


	// Web API does not really seem to let us check for readiness, other than the connection state.
	//
	fn poll_ready( self: Pin<&mut Self>, _: &mut Context ) -> Poll<Result<(), Self::Error>>
	{
		trace!( "Sink<WsMessage> for WsIo: poll_ready" );

		match self.ready_state()
		{
			WsState::Connecting => Poll::Pending        ,
			WsState::Open       => Poll::Ready( Ok(()) ),
			_                   => Poll::Ready( Err( WsErrKind::ConnectionClosed.into() )),
		}
	}


	fn start_send( self: Pin<&mut Self>, item: WsMessage ) -> Result<(), Self::Error>
	{
		trace!( "Sink<WsMessage> for WsIo: start_send" );

		match self.ready_state()
		{
			WsState::Open       =>
			{
				// TODO: fix the unwrap
				//
				match item
				{
					WsMessage::Binary( mut d ) => { self.ws.send_with_u8_array( &mut d ).unwrap(); }
					WsMessage::Text  (     s ) => { self.ws.send_with_str     ( &    s ).unwrap(); }
				}

				Ok(())
			},


			WsState::Connecting => Err( WsErrKind::ConnectionNotReady.into() ),

			// Closing or Closed
			//
			_ => Err( WsErrKind::ConnectionClosed.into() ),
		}
	}



	fn poll_flush( self: Pin<&mut Self>, _: &mut Context ) -> Poll<Result<(), Self::Error>>
	{
		trace!( "Sink<WsMessage> for WsIo: poll_flush" );

		Poll::Ready( Ok(()) )
	}



	// TODO: find a simpler implementation, notably this needs to clone the websocket and spawn a future.
	//
	fn poll_close( self: Pin<&mut Self>, cx: &mut Context ) -> Poll<Result<(), Self::Error>>
	{
		trace!( "Sink<WsMessage> for WsIo: poll_close" );

		let state = self.ready_state();


		if state == WsState::Connecting
		|| state == WsState::Open
		{
			self.ws.close().unwrap_throw();
		}


		match state
		{
			WsState::Closed =>
			{
				trace!( "WebSocket connection closed!" );
				Poll::Ready( Ok(()) )
			}

			_ =>
			{
				rt::spawn_local( Self::wake_on_close( self.ws.clone(), cx.waker().clone() ) ).expect( "spawn wake_on_close" );
				Poll::Pending
			}
		}
	}
}

