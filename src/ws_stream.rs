use
{
	crate :: { import::*, future_event, WsErr, WsErrKind, WsState, WsIo },
};


/// The meta data related to a websocket.
///
/// Most of the methods on this type directly map to the web API. For more documentation, check the
/// [MDN WebSocket documentation]((https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/WebSocket).
///
#[ derive( Clone ) ]
//
pub struct WsStream
{
	ws: Rc<WebSocket>
}



impl WsStream
{
	/// Connect to the server. The future will resolve when the connection has been established and the WebSocket
	/// handshake sucessful. There is no timeout mechanism here in case of failure. You should implement
	/// that yourself.
	///
	/// Can fail if there is a
	/// [security error](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/WebSocket#Exceptions_thrown).
	///
	/// This returns both a [WsStream] (allow manipulating and requesting metatdata for the connection) and
	/// a [WsIo] (Stream/Sink over [WsMessage] + AsyncRead/AsyncWrite).
	///
	/// **Note**: Sending protocols to a server that doesn't support them will make the connection fail.
	//
	pub async fn connect( url: impl AsRef<str>, protocols: impl Into<Option<Vec<&str>>> )

		-> Result< (Self, WsIo), WsErr >
	{
		let ws = Rc::new( match protocols.into()
		{
			None => WebSocket::new( url.as_ref() ).map_err( |_| WsErr::from( WsErrKind::SecurityError ) )?,

			Some(v) =>
			{
				let js_protos = v.iter().fold( Array::new(), |acc, proto|
				{
					acc.push( &JsValue::from_str( proto ) );
					acc
				});

				WebSocket::new_with_str_sequence( url.as_ref(), &js_protos )

					.map_err( |_| WsErr::from( WsErrKind::SecurityError ) )?
			}
		});


		ws.set_binary_type( BinaryType::Arraybuffer );

		future_event( |cb| ws.set_onopen( cb ) ).await;

		trace!( "WebSocket connection opened!" );

		Ok(( Self{ ws: ws.clone() }, WsIo::new( ws ) ))
	}



	/// Close the socket. The future will resolve once the socket's state has become `WsState::CLOSED`.
	/// See: [MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/close)
	//
	pub async fn close( &self )
	{
		// This can not throw normally, because the only errors the api
		// can return is if we use a code or a reason string, which we don't.
		// See [mdn](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/close#Exceptions_thrown).
		//
		self.ws.close().unwrap_throw();

		future_event( |cb| self.ws.set_onclose( cb ) ).await;

		trace!( "WebSocket connection closed!" );
	}



	/// Close the socket. The future will resolve once the socket's state has become `WsState::CLOSED`.
	/// See: [MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/close)
	//
	pub async fn close_code( &self, code: u16  ) -> Result<(), WsErr>
	{
		match self.ws.close_with_code( code )
		{
			Ok(_) =>
			{
				future_event( |cb| self.ws.set_onclose( cb ) ).await;

				trace!( "WebSocket connection closed!" );

				Ok(())
			}

			Err( _e ) =>
			{
				// TODO: figure out how to print the original error
				//
				// error!( "{}", e.as_string().expect( "JsValue to string" ) );
				//
				Err( WsErrKind::InvalidCloseCode( code ).into() )
			}
		}
	}



	/// Close the socket. The future will resolve once the socket's state has become `WsState::CLOSED`.
	/// See: [MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/close)
	//
	pub async fn close_reason( &self, code: u16, reason: impl AsRef<str>  ) -> Result<(), WsErr>
	{
		if reason.as_ref().len() > 123
		{
			return Err( WsErrKind::ReasonStringToLong.into() )
		}

		match self.ws.close_with_code_and_reason( code, reason.as_ref() )
		{
			Ok(_) =>
			{
				future_event( |cb| self.ws.set_onclose( cb ) ).await;

				trace!( "WebSocket connection closed!" );

				Ok(())
			}

			Err( _e ) =>
			{
				// TODO: figure out how to print the original error
				//
				// error!( "{}", e.as_string().expect( "JsValue to string" ) );
				//
				Err( WsErrKind::InvalidCloseCode(code).into() )
			}
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


	/// The number of bytes of data that have been queued but not yet transmitted to the network.
	///
	/// **NOTE:** that this is the number of bytes buffered by the underlying platform WebSocket
	/// implementation. It does not reflect any buffering performed by ws_stream.
	//
	pub fn buffered_amount( &self ) -> u32
	{
		self.ws.buffered_amount()
	}


	/// The extensions selected by the server as negotiated during the connection.
	///
	/// **NOTE**: This is an untested feature. The backend server we use for testing (tungstenite)
	/// does not support Extensions.
	//
	pub fn extensions( &self ) -> String
	{
		self.ws.extensions()
	}


	/// The name of the subprotocol the server selected during the connection.
	///
	/// This will be one of the strings specified in the protocols parameter when
	/// creating this WsStream instance.
	//
	pub fn protocol(&self) -> String
	{
		self.ws.protocol()
	}


	/// Retrieve the address to which this socket is connected.
	//
	pub fn url( &self ) -> String
	{
		self.ws.url()
	}
}



impl fmt::Debug for WsStream
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{
		write!( f, "WsStream for connection: {}", self.url() )
	}
}
