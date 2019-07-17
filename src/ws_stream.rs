use
{
	crate :: { import::*, future_event, WsErr, WsErrKind, WsState, WsIo, WsIoBinary },
};


/// The meta data related to a websocket.
///
/// Most of the methods on this type directly map to the web API. For more documentation, check the
/// [MDN WebSocket documentation]((https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/WebSocket).
///
//
pub struct WsStream
{
	ws: WebSocket
}



impl WsStream
{
	/// Connect to the server. The future will resolve when the connection has been established. There is currently
	/// no timeout mechanism here in case of failure. You should implement that yourself.
	///
	/// Can fail if there is a
	/// [security error](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/WebSocket#Exceptions_thrown).
	///
	pub async fn connect( url: impl AsRef<str>, protocols: impl Into<Option<Vec<&str>>> )

		-> Result< (Self, WsIo), WsErr >
	{
		let ws = match protocols.into()
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
		};


		ws.set_binary_type( BinaryType::Arraybuffer );

		future_event( |cb| ws.set_onopen( cb ) ).await;
		ws.set_onopen( None );                           // drop event handler

		trace!( "WebSocket connection opened!" );

		Ok(( Self{ ws: ws.clone() }, WsIo::new( ws ) ))
	}


	/// Connect to the server. The future will resolve when the connection has been established. There is currently
	/// no timeout mechanism here in case of failure. You should implement that yourself.
	///
	/// This creates a WsIoBinary, which is a stream over Vec<u8> and implements AsyncRead/AsyncWrite rather than
	/// a stream over JsMsgEvent.
	///
	/// Can fail if there is a
	/// [security error](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/WebSocket#Exceptions_thrown).
	//
	pub async fn connect_binary( url: impl AsRef<str>, protocols: impl Into<Option<Vec<&str>>> )

		-> Result< (Self, WsIoBinary), WsErr >
	{
		let (ws_stream, wsio) = Self::connect( url, protocols ).await?;

		Ok(( ws_stream, WsIoBinary::new( wsio ) ))
	}



	/// Close the socket. The future will resolve once the socket's state has become `WsReadyState::CLOSED`.
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



	/// Close the socket. The future will resolve once the socket's state has become `WsReadyState::CLOSED`.
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



	/// Close the socket. The future will resolve once the socket's state has become `WsReadyState::CLOSED`.
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



	/// Verify the [WsReadyState] of the connection.
	/// TODO: verify error handling
	//
	pub fn ready_state( &self ) -> WsState
	{
		self.ws.ready_state().try_into().map_err( |e| error!( "{}", e ) ).unwrap_throw()
	}


	/// Access the wrapped [web_sys::WebSocket](https://docs.rs/web-sys/0.3.25/web_sys/struct.WebSocket.html).
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
