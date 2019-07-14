use
{
	crate :: { import::*, future_event, WsState, WsIo, WsIoBinary },
};


/// The meta data related to a websocket
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
	pub async fn connect< T: AsRef<str> >( url: T ) -> Result< (Self, WsIo), JsValue >
	{
		let ws = WebSocket::new( url.as_ref() )?;

		future_event( |cb| ws.set_onopen( cb ) ).await;
		trace!( "WebSocket connection opened!" );

		Ok(( Self{ ws: ws.clone() }, WsIo::new( ws ) ))
	}


	/// Connect to the server. The future will resolve when the connection has been established. There is currently
	/// no timeout mechanism here in case of failure. You should implement that yourself.
	///
	pub async fn connect_binary< T: AsRef<str> >( url: T ) -> Result< (Self, WsIoBinary), JsValue >
	{
		let ws = WebSocket::new( url.as_ref() )?;

		future_event( |cb| ws.set_onopen( cb ) ).await;
		trace!( "WebSocket connection opened!" );

		Ok(( Self{ ws: ws.clone() }, WsIoBinary::new( ws ) ))
	}



	/// Close the socket. The future will resolve once the socket's state has become `WsReadyState::CLOSED`.
	///
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


	/// Retrieve the address to which this socket is connected.
	//
	pub fn url( &self ) -> String
	{
		self.ws.url()
	}
}
