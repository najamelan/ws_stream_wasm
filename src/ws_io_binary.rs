use
{
	crate :: { import::*, WsErr, WsErrKind, WsIo, WsMessage },
};


/// This implements Sink/Stream over Vec<u8> instead of WsMessage. It further implements AsyncRead/AsyncWrite
/// that can be framed with codecs. You can use the compat layer from the futures library if you want to
/// use tokio codecs. See the [integration tests](https://github.com/ws_stream_wasm/tree/master/tests/tokio_codec.rs)
/// if you need an example.
///
/// `WsIoBinary` can be created with [WsStream::connect_binary()].
//
pub struct WsIoBinary
{
	ws: WsIo,
}


impl WsIoBinary
{
	/// Create a new WsIoBinary. Can fail if there is a
	/// [security error](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/WebSocket#Exceptions_thrown).
	/// It is recommended to use [WsStream::connect_binary()] instead of this method.
	//
	pub fn new( ws: WsIo ) -> Self
	{
		Self { ws }
	}



	/// Access the wrapped [WsIo].
	//
	pub fn wrapped( &self ) -> &WsIo
	{
		&self.ws
	}



	/// Access the wrapped [WsIo] mutably.
	//
	pub fn wrapped_mut( &mut self ) -> &mut WsIo
	{
		&mut self.ws
	}
}



impl fmt::Debug for WsIoBinary
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{
		write!( f, "WsIoBinary for connection: {}", self.ws.wrapped().url() )
	}
}



impl fmt::Display for WsIoBinary
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{
		write!( f, "WsIoBinary for connection: {}", self.ws.wrapped().url() )
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

		match item.unwrap()
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
		let _ = ready!( Pin::new( &mut self.ws ).poll_close( cx ) );

		// WsIo poll_close is infallible
		//
		Poll::Ready( Ok(()) )
	}
}




