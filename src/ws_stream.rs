use
{
	crate :: { import::*, WsErr, WsErrKind, WsState, WsIo, WsEvent, CloseEvent, NextEvent, WsEventType, notify },
};


/// The meta data related to a websocket.
///
/// Most of the methods on this type directly map to the web API. For more documentation, check the
/// [MDN WebSocket documentation](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/WebSocket).
//
pub struct WsStream
{
	ws       : Rc<WebSocket>                      ,
	pharos   : Rc<RefCell< Pharos<WsEvent> >>     ,

	_on_open : Closure< dyn FnMut() >             ,
	_on_error: Closure< dyn FnMut() >             ,
	_on_close: Closure< dyn FnMut( JsCloseEvt ) > ,
}



impl WsStream
{
	/// Connect to the server. The future will resolve when the connection has been established with a successful WebSocket
	/// handshake.
	///
	/// This returns both a [WsStream] (allow manipulating and requesting metatdata for the connection) and
	/// a [WsIo] (AsyncRead/AsyncWrite + Stream/Sink over [WsMessage](crate::WsMessage)).
	///
	/// A WsStream instance is observable through the [`pharos::Observable`](https://docs.rs/pharos/0.2.0/pharos/trait.Observable.html) and [`pharos::ObservableUnbounded`](https://docs.rs/pharos/0.2.0/pharos/trait.UnboundedObservable.html) traits. The type of event is [WsEvent]. In the case of a Close event, there will be additional information included
	/// as a [CloseEvent].
	///
	/// When you drop this, the connection does not get closed, however when you drop [WsIo] it does. Streams
	/// of events will be dropped, so you will no longer receive events. One thing is possible if you really
	/// need it, that's dropping [WsStream] but keeping [WsIo]. Now through [WsIo::wrapped] you can
	/// access the underlying [web_sys::WebSocket] and set event handlers on it for `on_open`, `on_close`,
	/// `on_error`. If you would do that while [WsStream] is still around, that would break the event system
	/// and can lead to errors if you still call methods on [WsStream].
	///
	/// **Note**: Sending protocols to a server that doesn't support them will make the connection fail.
	///
	/// ## Errors
	///
	/// Browsers will forbid making websocket connections to certain ports. See this [Stack Overflow question](https://stackoverflow.com/questions/4313403/why-do-browsers-block-some-ports/4314070).
	/// `connect` will return a [WsErrKind::ForbiddenPort].
	///
	/// If the url is invalid, a [WsErrKind::InvalidUrl] is returned. See the [HTML Living Standard](https://html.spec.whatwg.org/multipage/web-sockets.html#dom-websocket) for more information.
	///
	/// When the connection fails (server port not open, wrong ip, wss:// on ws:// server, ... See the [HTML Living Standard](https://html.spec.whatwg.org/multipage/web-sockets.html#dom-websocket)
	/// for details on all failure possibilities), a [WsErrKind::ConnectionFailed] is returned.
	//
	pub async fn connect( url: impl AsRef<str>, protocols: impl Into<Option<Vec<&str>>> )

		-> Result< (Self, WsIo), WsErr >
	{
		let res = match protocols.into()
		{
			None => WebSocket::new( url.as_ref() ),

			Some(v) =>
			{
				let js_protos = v.iter().fold( Array::new(), |acc, proto|
				{
					acc.push( &JsValue::from_str( proto ) );
					acc
				});

				WebSocket::new_with_str_sequence( url.as_ref(), &js_protos )
			}
		};


		// Deal with errors from the WebSocket constructor
		//
		let ws = match res
		{
			Ok(ws) => Rc::new( ws ),

			Err(e) =>
			{
				let de: &DomException = e.unchecked_ref();

				match de.code()
				{
					DomException::SECURITY_ERR => return Err( WsErrKind::ForbiddenPort.into()                         ),
					DomException::SYNTAX_ERR   => return Err( WsErrKind::InvalidUrl( url.as_ref().to_string()).into() ),
					_                          => unreachable!(),
				};
			}
		};


		// Create our pharos
		//
		let pharos = Rc::new( RefCell::new( Pharos::new() ));
		let ph1    = pharos.clone();
		let ph2    = pharos.clone();
		let ph3    = pharos.clone();
		let ph4    = pharos.clone();


		// Setup our event listeners
		//
		let on_open = Closure::wrap( Box::new( move ||
		{
			trace!( "websocket open event" );

			// notify observers
			//
			notify( ph1.clone(), WsEvent::Open )


		}) as Box< dyn FnMut() > );


		let on_error = Closure::wrap( Box::new( move ||
		{
			trace!( "websocket error event" );

			// notify observers
			//
			notify( ph2.clone(), WsEvent::Error )


		}) as Box< dyn FnMut() > );


		let on_close = Closure::wrap( Box::new( move |evt: JsCloseEvt|
		{
			trace!( "websocket close event" );

			let c = WsEvent::Close( CloseEvent
			{
				code     : evt.code()     ,
				reason   : evt.reason()   ,
				was_clean: evt.was_clean(),
			});

			notify( ph3.clone(), c )


		}) as Box< dyn FnMut( JsCloseEvt ) > );


		ws.set_onopen ( Some( &on_open .as_ref().unchecked_ref() ));
		ws.set_onclose( Some( &on_close.as_ref().unchecked_ref() ));
		ws.set_onerror( Some( &on_error.as_ref().unchecked_ref() ));



		// Listen to the events to figure out whether the connection opens successfully. We don't want to deal with
		// the error event. Either a close event happens, in which case we want to recover the CloseEvent to return it
		// to the user, or an Open event happens in which case we are happy campers.
		//
		let evts = NextEvent::new( pharos.borrow_mut().observe_unbounded(), WsEventType::CLOSE | WsEventType::OPEN );


		// If the connection is closed, return error
		//
		if let Some( WsEvent::Close(evt) ) = evts.await
		{
			trace!( "WebSocket connection closed!" );

			return Err( WsErrKind::ConnectionFailed(evt).into() )
		}


		trace!( "WebSocket connection opened!" );

		// We don't handle Blob's
		//
		ws.set_binary_type( BinaryType::Arraybuffer );


		Ok
		((
			Self
			{
				pharos                ,
				ws       : ws.clone() ,
				_on_open : on_open    ,
				_on_error: on_error   ,
				_on_close: on_close   ,
			},

			WsIo::new( ws, ph4 )
		))
	}



	/// Close the socket. The future will resolve once the socket's state has become `WsState::CLOSED`.
	/// See: [MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/close)
	//
	pub async fn close( &self ) -> Result< CloseEvent, WsErr >
	{
		match self.ready_state()
		{
			WsState::Closed  => return Err( WsErrKind::ConnectionNotOpen.into() ),
			WsState::Closing => {}

			_ =>
			{
				// This can not throw normally, because the only errors the api can return is if we use a code or
				// a reason string, which we don't.
				// See [mdn](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/close#Exceptions_thrown).
				//
				self.ws.close().unwrap_throw();


				// Notify Observers
				//
				notify( self.pharos.clone(), WsEvent::Closing )
			}
		}


		let evts = NextEvent::new( self.pharos.borrow_mut().observe_unbounded(), WsEventType::CLOSE );


		// We promised the user a CloseEvent, so we don't have much choice but to unwrap this. In any case, the stream will
		// never end and this will hang if the browser fails to send a close event.
		//
		let ce = evts.await.expect_throw( "receive a close event" );
		trace!( "WebSocket connection closed!" );

		if let WsEvent::Close(e) = ce { Ok( e )        }
		else                          { unreachable!() }
	}




	/// Close the socket. The future will resolve once the socket's state has become `WsState::CLOSED`.
	/// See: [MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/close)
	//
	pub async fn close_code( &self, code: u16  ) -> Result<CloseEvent, WsErr>
	{
		match self.ready_state()
		{
			WsState::Closed  => return Err( WsErrKind::ConnectionNotOpen.into() ),
			WsState::Closing => {}

			_ =>
			{
				match self.ws.close_with_code( code )
				{
					// Notify Observers
					//
					Ok(_) => notify( self.pharos.clone(), WsEvent::Closing ),


					Err(_) =>
					{
						let e = WsErr::from( WsErrKind::InvalidCloseCode( code ) );

						error!( "{}", e );

						return Err( e );
					}
				}
			}
		}


		let evts = NextEvent::new( self.pharos.borrow_mut().observe_unbounded(), WsEventType::CLOSE );

		let ce = evts.await.expect_throw( "receive a close event" );
		trace!( "WebSocket connection closed!" );

		if let WsEvent::Close(e) = ce { Ok(e)          }
		else                          { unreachable!() }
	}



	/// Close the socket. The future will resolve once the socket's state has become `WsState::CLOSED`.
	/// See: [MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/close)
	//
	pub async fn close_reason( &self, code: u16, reason: impl AsRef<str>  ) -> Result<CloseEvent, WsErr>
	{
		match self.ready_state()
		{
			WsState::Closed  => return Err( WsErrKind::ConnectionNotOpen.into() ),
			WsState::Closing => {}

			_ =>
			{
				if reason.as_ref().len() > 123
				{
					let e = WsErr::from( WsErrKind::ReasonStringToLong );

					error!( "{}", e );

					return Err( e );
				}


				match self.ws.close_with_code_and_reason( code, reason.as_ref() )
				{
					// Notify Observers
					//
					Ok(_) => notify( self.pharos.clone(), WsEvent::Closing ),


					Err(_) =>
					{
						let e = WsErr::from( WsErrKind::InvalidCloseCode( code ) );

						error!( "{}", e );

						return Err( e )
					}
				}
			}
		}

		let evts = NextEvent::new( self.pharos.borrow_mut().observe_unbounded(), WsEventType::CLOSE );

		let ce = evts.await.expect_throw( "receive a close event" );
		trace!( "WebSocket connection closed!" );

		if let WsEvent::Close(e) = ce { Ok(e)          }
		else                          { unreachable!() }
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



impl Observable<WsEvent> for WsStream
{
	fn observe( &mut self, queue_size: usize ) -> Receiver<WsEvent>
	{
		self.pharos.borrow_mut().observe( queue_size )
	}
}



impl UnboundedObservable<WsEvent> for WsStream
{
	fn observe_unbounded( &mut self ) -> UnboundedReceiver<WsEvent>
	{
		self.pharos.borrow_mut().observe_unbounded()
	}
}



impl Drop for WsStream
{
	// We don't block here, just tell the browser to close the connection and move on.
	// TODO: is this necessary or would it be closed automatically when we drop the WebSocket
	// object? Note that there is also the WsStream which holds a clone.
	//
	fn drop( &mut self )
	{
		trace!( "Drop WsStream" );

		self.ws.set_onopen ( None );
		self.ws.set_onclose( None );
		self.ws.set_onerror( None );
	}
}


