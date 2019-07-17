use
{
	crate :: { import::*, WsErr, WsErrKind, WsIo, WsMessage },
};


/// A wrapper around [web_sys::WebSocket](https://docs.rs/web-sys/0.3.25/web_sys/struct.WebSocket.html) to make it more rust idiomatic.
/// It does not provide any extra functionality over the wrapped WebSocket object.
///
/// It turns the callback based mechanisms into futures Sink and Stream. The stream yields [JsMsgEvent], which is a wrapper
/// around [`web_sys::MessageEvent`](https://docs.rs/web-sys/0.3.25/web_sys/struct.MessageEvent.html) and the sink takes a
/// [WsMessage] which is a wrapper around  [`web_sys::MessageEvent.data()`](https://docs.rs/web-sys/0.3.25/web_sys/struct.MessageEvent.html#method.data).
/// There is no error when the server is not running, and no timeout mechanism provided here to detect that connection
/// never happens. The connect future will just never resolve.
///
/// ## Example
///
/// ```
/// #![ feature( async_await, await_macro, futures_api )]
///
/// use
/// {
///    futures::prelude      ::* ,
///    wasm_bindgen::prelude ::* ,
///    wasm_bindgen_futures  ::* ,
///    wasm_websocket_stream ::* ,
///    log                   ::* ,
/// };
///
/// let fut = async
/// {
///    let ws = WsIoBinary::new( URL ).expect_throw( "Could not create websocket" );
///
///    ws.connect().await;
///
///    let (mut tx, mut rx) = ws.split();
///    let message          = "Hello from browser".to_string();
///
///
///    tx.send( WsMessage::Text( message.clone() )).await
///
///       .expect_throw( "Failed to write to websocket" );
///
///
///    let msg    = rx.next().await;
///    let result = &msg.expect_throw( "Stream closed" );
///
///    assert_eq!( WsMessage::Text( message ), result.data() );
///
///    Ok(())
///
/// }.boxed().compat();
///
/// spawn_local( fut );
/// ```
///
#[ allow( dead_code ) ] // we keep the closure to keep it form being dropped
//
pub struct WsIoBinary
{
	ws: WsIo,
}


impl WsIoBinary
{
	/// Create a new WsIoBinary. Can fail if there is a
	/// [security error](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/WebSocket#Exceptions_thrown).
	///
	pub fn new( ws: WebSocket ) -> Self
	{
		Self { ws: WsIo::new( ws ) }
	}


	/// Create a new WsIoBinary with the callback for received messages. Can fail if there is a
	/// [security error](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/WebSocket#Exceptions_thrown).
	///
	pub fn with_on_message< T: AsRef<str> >( url: T, onmesg: Box< dyn FnMut( MessageEvent ) > ) -> Result< Self, JsValue >
	{
		let ws = WsIo::with_on_message( url, onmesg );

		match ws
		{
			Ok (ws) => Ok( Self { ws } ) ,
			Err(e ) => Err( e )          ,
		}
	}


	/// Access the wrapped [web_sys::WebSocket](https://docs.rs/web-sys/0.3.25/web_sys/struct.WebSocket.html).
	///
	pub fn wrapped( &self ) -> &WsIo
	{
		&self.ws
	}
}



impl fmt::Debug for WsIoBinary
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{
		write!( f, "WsIoBinary" )
	}
}



impl fmt::Display for WsIoBinary
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{
		write!( f, "WsIoBinary" )
	}
}



impl Stream for WsIoBinary
{
	type Item = Result< Vec<u8>, std::io::Error >;

	// Forward the call to the channel on which we are listening.
	//
	// Currently requires an unfortunate copy from Js memory to Wasm memory. Hopefully one
	// day we will be able to receive the JsMsgEvent directly in Wasm.
	//
	fn poll_next( mut self: Pin<&mut Self>, cx: &mut std::task::Context ) -> Poll<Option< Self::Item >>
	{
		trace!( "WsIoBinary as Stream gets polled" );

		let item = ready!( Pin::new( &mut self.ws ).poll_next( cx ) );

		if item.is_none()
		{
			return Poll::Ready( None );
		}

		match item.unwrap().data()
		{
			WsMessage::Text  ( string ) => Poll::Ready(Some(Ok( string.into() ))),
			WsMessage::Binary( chunk  ) => Poll::Ready(Some(Ok( chunk         ))),
		}
	}
}





impl Sink<Vec<u8>> for WsIoBinary
{
	type Error = WsErr;


	// Web api does not really seem to let us check for readiness, other than the connection state.
	//
	fn poll_ready( mut self: Pin<&mut Self>, cx: &mut std::task::Context ) -> Poll<Result<(), Self::Error>>
	{
		Pin::new( &mut self.ws ).poll_ready( cx )
	}


	fn start_send( mut self: Pin<&mut Self>, item: Vec<u8> ) -> Result<(), Self::Error>
	{
		Pin::new( &mut self.ws ).start_send( WsMessage::Binary( item ) )
	}



	fn poll_flush( mut self: Pin<&mut Self>, cx: &mut std::task::Context ) -> Poll<Result<(), Self::Error>>
	{
		Pin::new( &mut self.ws ).poll_flush( cx )

	}



	fn poll_close( mut self: Pin<&mut Self>, cx: &mut std::task::Context ) -> Poll<Result<(), Self::Error>>
	{
		Pin::new( &mut self.ws ).poll_close( cx )
	}
}





impl AsyncWrite for WsIoBinary
{
	fn poll_write( mut self: Pin<&mut Self>, cx: &mut Context, buf: &[u8] ) -> Poll<Result<usize, io::Error>>
	{
		let res = ready!( self.as_mut().poll_ready( cx ) );

		match res
		{
			Ok(_) =>
			{
				let n = buf.len();

				match self.start_send( buf.into() )
				{
					Ok (_) => { return Poll::Ready( Ok(n) ); }
					Err(e) =>
					{
						match e.kind()
						{
							WsErrKind::ConnectionClosed => { return Poll::Ready( Err( io::Error::from( io::ErrorKind::NotConnected ))) }
							_                           => unreachable!()
						}
					}
				}
			}

			Err(e) => match e.kind()
			{
				WsErrKind::ConnectionClosed => { return Poll::Ready( Err( io::Error::from( io::ErrorKind::NotConnected ))) }
				_                           => unreachable!()
			}
		}
	}



	fn poll_flush( self: Pin<&mut Self>, _cx: &mut Context ) -> Poll<Result<(), io::Error>>
	{
		Poll::Ready( Ok(()) )
	}


	fn poll_close( mut self: Pin<&mut Self>, cx: &mut Context ) -> Poll<Result<(), io::Error>>
	{
		// TODO: fix the unwrap once web-sys can return errors: https://github.com/rustwasm/wasm-bindgen/issues/1286
		//
		let _ = Pin::new( &mut self.ws ).poll_close(cx);


		// it's infallible for now
		//
		Poll::Ready( Ok(()) )
	}
}




