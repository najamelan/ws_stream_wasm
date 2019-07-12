use
{
	crate :: { import::*, WsErr, WsErrKind, JsMsgEvent, JsMsgEvtData, future_event },
};


/// A wrapper around [web_sys::WebSocket](https://docs.rs/web-sys/0.3.17/web_sys/struct.WebSocket.html) to make it more rust idiomatic.
/// It does not provide any extra functionality over the wrapped WebSocket object.
///
/// It turns the callback based mechanisms into futures Sink and Stream. The stream yields [JsMsgEvent], which is a wrapper
/// around [`web_sys::MessageEvent`](https://docs.rs/web-sys/0.3.17/web_sys/struct.MessageEvent.html) and the sink takes a
/// [JsMsgEvtData] which is a wrapper around  [`web_sys::MessageEvent.data()`](https://docs.rs/web-sys/0.3.17/web_sys/struct.MessageEvent.html#method.data).
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
///    let ws = JsWebSocket::new( URL ).expect_throw( "Could not create websocket" );
///
///    ws.connect().await;
///
///    let (mut tx, mut rx) = ws.split();
///    let message          = "Hello from browser".to_string();
///
///
///    tx.send( JsMsgEvtData::Text( message.clone() )).await
///
///       .expect_throw( "Failed to write to websocket" );
///
///
///    let msg    = rx.next().await;
///    let result = &msg.expect_throw( "Stream closed" );
///
///    assert_eq!( JsMsgEvtData::Text( message ), result.data() );
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
pub struct JsWebSocket
{
	ws     : WebSocket                                      ,
	on_mesg: Closure< dyn FnMut( MessageEvent ) + 'static > ,
	queue  : Rc<RefCell< VecDeque<JsMsgEvent> >>            ,
	task   : Rc<RefCell< Option<task::Task>   >>            ,
}


impl JsWebSocket
{
	/// Create a new JsWebSocket. Can fail if there is a
	/// [security error](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/WebSocket#Exceptions_thrown).
	///
	pub fn new< T: AsRef<str> >( url: T ) -> Result< Self, JsValue >
	{
		let task: Rc<RefCell<Option<task::Task>>> = Rc::new( RefCell::new( None ) );

		let queue = Rc::new( RefCell::new( VecDeque::new() ) );
		let q2    = queue.clone();
		let t2    = task .clone();
		let ws    = WebSocket::new( url.as_ref() )?;


		// Send the incoming ws messages to the WsStream object
		//
		let on_mesg = Closure::wrap( Box::new( move |msg_evt: MessageEvent|
		{
			trace!( "WsStream: message received!" );

			#[ cfg( debug_assertions )]
			//
			dbg( &msg_evt );

			q2.borrow_mut().push_back( JsMsgEvent{ msg_evt } );

			if let Some( ref t ) = *t2.borrow()
			{
				trace!( "WsStream: waking up task" );
				t.notify()
			}

		}) as Box< dyn FnMut( MessageEvent ) > );


		// Install callback
		//
		ws.set_onmessage  ( Some( on_mesg.as_ref().unchecked_ref() ) );
		ws.set_binary_type( BinaryType::Arraybuffer                  );


		Ok( Self
		{
			ws       ,
			queue    ,
			on_mesg  ,
			task     ,
		})
	}


	/// Create a new JsWebSocket with the callback for received messages. Can fail if there is a
	/// [security error](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/WebSocket#Exceptions_thrown).
	///
	pub fn with_on_message< T: AsRef<str> >( url: T, onmesg: Box< dyn FnMut( MessageEvent ) > ) -> Result< Self, JsValue >
	{
		//  Internal note, in this case self.queue and self.task wont be used.
		//
		let task: Rc<RefCell<Option<task::Task>>> = Rc::new( RefCell::new( None ) );

		let queue   = Rc::new( RefCell::new( VecDeque::new() ) );
		let ws      = WebSocket::new( url.as_ref() )?;

		let on_mesg = Closure::wrap( onmesg );

		// Install callback
		//
		ws.set_onmessage  ( Some( on_mesg.as_ref().unchecked_ref() ) );
		ws.set_binary_type( BinaryType::Arraybuffer                  );

		Ok( Self
		{
			ws       ,
			queue    ,
			on_mesg  ,
			task     ,
		})
	}


	/// Connect to the server. The future will resolve when the connection has been established. There is currently
	/// no timeout mechanism here in case of failure. You should implement that yourself.
	///
	pub async fn connect( &self )
	{
		// FIXME: Can we just pass the function without having to run a closure here? When I tried this didn't work, because
		//        rustc complained: `Attempted to take value of method 'set_onopen' on type 'web_sys::WebSocket'`
		//
		future_event( |cb| self.ws.set_onopen( cb ) ).await;

		trace!( "WebSocket connection opened!" );

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
	///
	pub fn ready_state( &self ) -> WsReadyState
	{
		self.ws.ready_state().try_into().map_err( |e| error!( "{}", e ) ).unwrap_throw()
	}


	/// Access the wrapped [web_sys::WebSocket](https://docs.rs/web-sys/0.3.17/web_sys/struct.WebSocket.html).
	///
	pub fn wrapped( &self ) -> &WebSocket
	{
		&self.ws
	}


	/// Retrieve the address to which this socket is connected.
	///
	pub fn url( &self ) -> String
	{
		self.ws.url()
	}
}



impl fmt::Debug for JsWebSocket
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{
		write!( f, "JsWebSocket connected to: {}", self.url() )
	}
}



impl fmt::Display for JsWebSocket
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{
		write!( f, "JsWebSocket connected to: {}", self.url() )
	}
}



impl Stream for JsWebSocket
{
	type Item = JsMsgEvent;

	// Forward the call to the channel on which we are listening.
	//
	// Currently requires an unfortunate copy from Js memory to Wasm memory. Hopefully one
	// day we will be able to receive the JsMsgEvent directly in Wasm.
	//
	fn poll_next( mut self: Pin<&mut Self>, _: &mut std::task::Context ) -> Poll<Option< Self::Item >>
	{
		trace!( "JsWebSocket as Stream gets polled" );

		if self.queue.borrow().is_empty()
		{
			*self.task.borrow_mut() = Some( task::current() );

			match self.ready_state()
			{
				WsReadyState::OPEN       => Poll::Pending        ,
				WsReadyState::CONNECTING => Poll::Pending        ,
				_                        => Poll::Ready  ( None ),

			}
		}

		else { Poll::Ready( self.queue.borrow_mut().pop_front() ) }
	}
}





impl Sink<JsMsgEvtData> for JsWebSocket
{
	type Error = JsValue;


	fn poll_ready( self: Pin<&mut Self>, _: &mut std::task::Context ) -> Poll<Result<(), Self::Error>>
	{
		trace!( "Sink: Websocket ready to poll" );

		Poll::Ready( Ok(()) )
	}


	fn start_send( self: Pin<&mut Self>, item: JsMsgEvtData ) -> Result<(), Self::Error>
	{
		trace!( "Sink: start_send" );

		match item
		{
			JsMsgEvtData::Binary( mut d ) => self.ws.send_with_u8_array( &mut d ),
			JsMsgEvtData::Text  (     s ) => self.ws.send_with_str     ( &    s ),
		}
	}



	fn poll_flush( self: Pin<&mut Self>, _: &mut std::task::Context ) -> Poll<Result<(), Self::Error>>
	{
		trace!( "Sink: poll_flush" );

		Poll::Ready( Ok(()) )
	}



	fn poll_close( self: Pin<&mut Self>, _: &mut std::task::Context ) -> Poll<Result<(), Self::Error>>
	{
		trace!( "Sink: poll_close" );

		Poll::Ready( self.ws.close() )
	}
}






/// Indicates the state of a Websocket connection. The only state in which it's valid to send and receive messages
/// is OPEN.
///
/// See [MDN](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/readyState) for the ready state values.
///
#[ allow( missing_docs ) ]
//
#[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
//
pub enum WsReadyState
{
	CONNECTING,
	OPEN      ,
	CLOSING   ,
	CLOSED    ,
}


/// Internally ready state is a u16, so it's possible to create one from a u16. Only 0-3 are valid values.
///
/// See [MDN](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/readyState) for the ready state values.
///
impl TryFrom<u16> for WsReadyState
{
	type Error = WsErr;

	fn try_from( state: u16 ) -> Result< Self, Self::Error >
	{
		match state
		{
			0 => Ok ( WsReadyState::CONNECTING                     ) ,
			1 => Ok ( WsReadyState::OPEN                           ) ,
			2 => Ok ( WsReadyState::CLOSING                        ) ,
			3 => Ok ( WsReadyState::CLOSED                         ) ,
			_ => Err( WsErrKind::InvalidReadyState( state ).into() ) ,
		}
	}
}
