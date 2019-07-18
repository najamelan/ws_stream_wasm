use
{
	crate :: { import::*, WsErr, WsErrKind, WsMessage, WsState, future_event },
};


/// Sink/Stream of [WsMessage]. It further implements AsyncRead/AsyncWrite
/// that can be framed with codecs. You can use the compat layer from the futures library if you want to
/// use tokio codecs. See the [integration tests](https://github.com/ws_stream_wasm/tree/master/tests/tokio_codec.rs)
/// if you need an example.
///
/// Created with [WsStream::connect](crate::WsStream::connect).
///
#[ allow( dead_code ) ] // we need to store the closure to keep it form being dropped
//
pub struct WsIo
{
	ws     : Rc< WebSocket >                                ,
	on_mesg: Closure< dyn FnMut( MessageEvent ) + 'static > ,
	queue  : Rc<RefCell< VecDeque<WsMessage> >>             ,
	waker  : Rc<RefCell<Option<Waker>>>                     ,
	state  : ReadState                                      ,
}


impl WsIo
{
	/// Create a new WsIo.
	//
	pub fn new( ws: Rc<WebSocket> ) -> Self
	{
		let waker: Rc<RefCell<Option<Waker>>> = Rc::new( RefCell::new( None ));

		let state = ReadState::PendingChunk;
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
			state   ,
			waker   ,
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



impl Drop for WsIo
{
	// We don't block here, just tell the browser to close the connection and move on.
	// TODO: is this necessary or would it be closed automatically when we drop the WebSocket
	// object? Note that there is also the WsStream which holds a clone.
	//
	fn drop( &mut self )
	{
		trace!( "Drop WsIo" );

		// This can't fail
		//
		self.ws.close_with_code( 1000 ).expect( "WsIo::drop - close ws socket" );
	}
}



impl Stream for WsIo
{
	type Item = WsMessage;

	// Currently requires an unfortunate copy from Js memory to Wasm memory. Hopefully one
	// day we will be able to receive the MessageEvt directly in Wasm.
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
			_                   => Poll::Ready( Err( WsErrKind::ConnectionNotOpen.into() )),
		}
	}


	fn start_send( self: Pin<&mut Self>, item: WsMessage ) -> Result<(), Self::Error>
	{
		trace!( "Sink<WsMessage> for WsIo: start_send" );

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
					WsMessage::Binary( mut d ) => { self.ws.send_with_u8_array( &mut d ).map_err( |_| WsErrKind::ConnectionNotOpen)?; }
					WsMessage::Text  (     s ) => { self.ws.send_with_str     ( &    s ).map_err( |_| WsErrKind::ConnectionNotOpen)?; }
				}

				Ok(())
			},


			// Connecting, Closing or Closed
			//
			_ => Err( WsErrKind::ConnectionNotOpen.into() ),
		}
	}



	fn poll_flush( self: Pin<&mut Self>, _: &mut Context ) -> Poll<Result<(), Self::Error>>
	{
		trace!( "Sink<WsMessage> for WsIo: poll_flush" );

		Poll::Ready( Ok(()) )
	}



	// TODO: find a simpler implementation, notably this needs to spawn a future.
	//       this can be done by creating a custom future. If we are going to implement
	//       events with pharos, that's probably a good time to re-evaluate this.
	//
	fn poll_close( self: Pin<&mut Self>, cx: &mut Context ) -> Poll<Result<(), Self::Error>>
	{
		trace!( "Sink<WsMessage> for WsIo: poll_close" );

		let state = self.ready_state();


		if state == WsState::Connecting
		|| state == WsState::Open
		{
			// Can't fail
			//
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
				// Spawning on wasm is infallible
				//
				rt::spawn_local( Self::wake_on_close( self.ws.clone(), cx.waker().clone() ) ).expect( "spawn wake_on_close" );
				Poll::Pending
			}
		}
	}
}





impl AsyncWrite for WsIo
{
	fn poll_write( mut self: Pin<&mut Self>, cx: &mut Context, buf: &[u8] ) -> Poll<Result<usize, io::Error>>
	{
		let res = ready!( self.as_mut().poll_ready( cx ) );

		match res
		{
			Ok(_) =>
			{
				let n = buf.len();

				match self.start_send( WsMessage::Binary( buf.into() ) )
				{
					Ok (_) => { return Poll::Ready( Ok(n) ); }
					Err(e) =>
					{
						match e.kind()
						{
							WsErrKind::ConnectionNotOpen =>
							{
								return Poll::Ready( Err( io::Error::from( io::ErrorKind::NotConnected )))
							}

							// This shouldn't happen, so panic for early detection.
							//
							_ => unreachable!()
						}
					}
				}
			}

			Err(e) => match e.kind()
			{
				WsErrKind::ConnectionNotOpen =>
				{
					return Poll::Ready( Err( io::Error::from( io::ErrorKind::NotConnected )))
				}

				_ => unreachable!()
			}
		}
	}



	fn poll_flush( self: Pin<&mut Self>, _cx: &mut Context ) -> Poll<Result<(), io::Error>>
	{
		Poll::Ready( Ok(()) )
	}


	fn poll_close( self: Pin<&mut Self>, cx: &mut Context ) -> Poll<Result<(), io::Error>>
	{
		let _ = ready!( < Self as Sink<WsMessage> >::poll_close( self, cx ) );

		// WsIo poll_close is infallible
		//
		Poll::Ready( Ok(()) )
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
	fn poll_read( mut self: Pin<&mut Self>, cx: &mut Context, buf: &mut [u8] ) -> Poll< Result<usize, io::Error> >
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

					buf[..len].copy_from_slice( &chunk[*chunk_start..end] );


					if chunk.len() == end
					{
						self.state = ReadState::PendingChunk;
					}

					else
					{
						*chunk_start = end;
					}


					return Poll::Ready( Ok(len) );
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
							return Poll::Ready( Ok(0) );
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



