#![ feature( trait_alias )]
wasm_bindgen_test_configure!(run_in_browser);


// What's tested:
//
// Tests send to an echo server which just bounces back all data.
//
// ✔ WsStream::connect: Verify error when connecting to a wrong port
// ✔ WsStream::connect: Verify error when connecting to a forbidden port
// ✔ WsStream::connect: Verify error when connecting to wss:// on ws:// server
// ✔ WsStream::connect: Verify error when connecting to a wrong scheme
// ✔ Verify the state method
// ✔ Verify closing from WsIo
// ✔ Verify url method
// ✔ Verify sending no subprotocols
//   note: we currently don't have a backend server that supports protocols,
//   so there is no test for testing usage of protocols
// ✔ Verify closing with a valid code
// ✔ Verify error upon closing with invalid code
// ✔ Verify closing with a valid code and reason
// ✔ Verfiy close_reason with an invalid close code
// ✔ Verfiy close_reason with an invalid reason string
// ✔ Verfiy Debug impl
//
use
{
	futures_01            :: { Future as Future01 } ,
	futures::prelude      :: { *                  } ,
	futures               :: { sink::SinkExt      } ,
	wasm_bindgen::prelude :: { *                  } ,
	wasm_bindgen_test     :: { *                  } ,
	ws_stream_wasm        :: { *                  } ,
	log                   :: { *                  } ,
};



const URL: &str = "ws://127.0.0.1:3212/";



// WsStream::connect: Verify error when connecting to a wrong port
//
#[ wasm_bindgen_test(async) ]
//
pub fn connect_wrong_port() -> impl Future01<Item = (), Error = JsValue>
{
	let _ = console_log::init_with_level( Level::Trace );

	info!( "starting test: connect_wrong_port" );

	async
	{
		let err = WsStream::connect( "ws://127.0.0.1:33212/", None ).await;

		assert!( err.is_err() );

		let err = err.unwrap_err();

		assert_eq!
		(
			&WsErrKind::ConnectionFailed
			(
				CloseEvent
				{
					was_clean: false,
					code     : 1006 ,
					reason   : "".to_string(),
				}
			),
			err.kind()
		);

		Ok(())

	}.boxed_local().compat()
}



// WsStream::connect: Verify error when connecting to a forbidden port
//
#[ wasm_bindgen_test(async) ]
//
pub fn connect_forbidden_port() -> impl Future01<Item = (), Error = JsValue>
{
	let _ = console_log::init_with_level( Level::Trace );

	info!( "starting test: connect_forbidden_port" );

	async
	{
		let err = WsStream::connect( "ws://127.0.0.1:6666/", None ).await;

		assert!( err.is_err() );

		let err = err.unwrap_err();

		assert_eq!( &WsErrKind::ForbiddenPort, err.kind() );


		Ok(())

	}.boxed_local().compat()
}



// WsStream::connect: Verify error when connecting to wss:// on ws:// server
//
#[ wasm_bindgen_test(async) ]
//
pub fn connect_wrong_wss() -> impl Future01<Item = (), Error = JsValue>
{
	let _ = console_log::init_with_level( Level::Trace );

	info!( "starting test: connect_wrong_wss" );

	async
	{
		let err = WsStream::connect( "wss://127.0.0.1:3212/", None ).await;

		assert!( err.is_err() );

		let err = err.unwrap_err();

		assert_eq!
		(
			&WsErrKind::ConnectionFailed
			(
				CloseEvent
				{
					was_clean: false,
					code     : 1006 ,
					reason   : "".to_string(),
				}
			),
			err.kind()
		);

		Ok(())

	}.boxed_local().compat()
}



// WsStream::connect: Verify error when connecting to a wrong scheme
//
#[ wasm_bindgen_test(async) ]
//
pub fn connect_wrong_scheme() -> impl Future01<Item = (), Error = JsValue>
{
	let _ = console_log::init_with_level( Level::Trace );

	info!( "starting test: connect_wrong_scheme" );

	async
	{
		let err = WsStream::connect( "http://127.0.0.1:3212/", None ).await;

		assert!( err.is_err() );

		let err = err.unwrap_err();

		assert_eq!( &WsErrKind::InvalidUrl( "http://127.0.0.1:3212/".to_string() ), err.kind() );

		Ok(())

	}.boxed_local().compat()
}



// Verify state method.
//
#[ wasm_bindgen_test(async) ]
//
pub fn state() -> impl Future01<Item = (), Error = JsValue>
{
	let _ = console_log::init_with_level( Level::Trace );

	info!( "starting test: state" );

	async
	{
		let (ws, wsio) = WsStream::connect( URL, None ).await.expect_throw( "Could not create websocket" );

		assert_eq!( WsState::Open, ws  .ready_state() );
		assert_eq!( WsState::Open, wsio.ready_state() );

		ws.wrapped().close().expect_throw( "close WebSocket" );

		assert_eq!( WsState::Closing, ws  .ready_state() );
		assert_eq!( WsState::Closing, wsio.ready_state() );

		ws.close().await.expect_throw( "close ws" );

		assert_eq!( WsState::Closed, ws  .ready_state() );
		assert_eq!( WsState::Closed, wsio.ready_state() );

		Ok(())

	}.boxed_local().compat()
}


// Verify closing from WsIo.
//
#[ wasm_bindgen_test(async) ]
//
pub fn close_from_wsio() -> impl Future01<Item = (), Error = JsValue>
{
	let _ = console_log::init_with_level( Level::Trace );

	info!( "starting test: close_from_wsio" );

	async
	{
		let (ws, mut wsio) = WsStream::connect( URL, None ).await.expect_throw( "Could not create websocket" );

		assert_eq!( WsState::Open, ws.ready_state() );

		SinkExt::close( &mut wsio ).await.expect( "close wsio sink" );

		assert_eq!( WsState::Closed, wsio.ready_state() );
		assert_eq!( WsState::Closed, ws  .ready_state() );

		Ok(())

	}.boxed_local().compat()
}




// Verify url method.
//
#[ wasm_bindgen_test(async) ]
//
pub fn url() -> impl Future01<Item = (), Error = JsValue>
{
	let _ = console_log::init_with_level( Level::Trace );

	info!( "starting test: url" );

	async
	{
		let (ws, _wsio) = WsStream::connect( URL, None ).await.expect_throw( "Could not create websocket" );

		assert_eq!( URL, ws.url() );

		Ok(())

	}.boxed_local().compat()
}




// Verify protocols.
//
#[ wasm_bindgen_test(async) ]
//
pub fn no_protocols() -> impl Future01<Item = (), Error = JsValue>
{
	let _ = console_log::init_with_level( Level::Trace );

	info!( "starting test: no_protocols" );

	async
	{
		let (ws, _wsio) = WsStream::connect( URL, None ).await.expect_throw( "Could not create websocket" );

		assert_eq!( "", ws.protocol() );

		Ok(())

	}.boxed_local().compat()
}




/*
// Verify protocols.
// This doesn't work with tungstenite for the moment.
//
#[ wasm_bindgen_test(async) ]
//
pub fn protocols_server_accept_none() -> impl Future01<Item = (), Error = JsValue>
{
	let _ = console_log::init_with_level( Level::Trace );

	info!( "starting test: protocols_server_accept_none" );

	async
	{
		let (ws, _wsio) = WsStream::connect( URL, vec![ "chat" ] ).await.expect_throw( "Could not create websocket" );

		assert_eq!( "", ws.protocol() );

		let r: Result<(), wasm_bindgen::JsValue> = Ok(());

		r

	}.boxed_local().compat()
}
*/



// Verify close_code method.
//
#[ wasm_bindgen_test(async) ]
//
pub fn close_twice() -> impl Future01<Item = (), Error = JsValue>
{
	let _ = console_log::init_with_level( Level::Trace );

	info!( "starting test: close_twice" );

	async
	{
		let (ws, _wsio) = WsStream::connect( URL, None ).await.expect_throw( "Could not create websocket" );

		let res = ws.close().await;

		assert!( res.is_ok() );

		assert_eq!( ws.close       ()                         .await.unwrap_err().kind(), &WsErrKind::ConnectionNotOpen );
		assert_eq!( ws.close_code  ( 1000                    ).await.unwrap_err().kind(), &WsErrKind::ConnectionNotOpen );
		assert_eq!( ws.close_reason( 1000, "Normal shutdown" ).await.unwrap_err().kind(), &WsErrKind::ConnectionNotOpen );

		Ok(())

	}.boxed_local().compat()
}



#[ wasm_bindgen_test(async) ]
//
pub fn close_code_valid() -> impl Future01<Item = (), Error = JsValue>
{
	let _ = console_log::init_with_level( Level::Trace );

	info!( "starting test: close_code_valid" );

	async
	{
		let (ws, _wsio) = WsStream::connect( URL, None ).await.expect_throw( "Could not create websocket" );

		let res = ws.close_code( 1000 ).await;

		assert!( res.is_ok() );

		Ok(())

	}.boxed_local().compat()
}


// Verify close_code method.
//
#[ wasm_bindgen_test(async) ]
//
pub fn close_code_invalid() -> impl Future01<Item = (), Error = JsValue>
{
	let _ = console_log::init_with_level( Level::Trace );

	info!( "starting test: close_code_invalid" );

	async
	{
		let (ws, _wsio) = WsStream::connect( URL, None ).await.expect_throw( "Could not create websocket" );

		let res = ws.close_code( 500 ).await;

		assert_eq!( &WsErrKind::InvalidCloseCode(500), res.unwrap_err().kind() );

		Ok(())

	}.boxed_local().compat()
}


// Verify close_code method.
//
#[ wasm_bindgen_test(async) ]
//
pub fn close_reason_valid() -> impl Future01<Item = (), Error = JsValue>
{
	let _ = console_log::init_with_level( Level::Trace );

	info!( "starting test: close_reason_valid" );

	async
	{
		let (ws, _wsio) = WsStream::connect( URL, None ).await.expect_throw( "Could not create websocket" );

		let res = ws.close_reason( 1000, "Normal shutdown" ).await;

		assert!( res.is_ok() );

		Ok(())

	}.boxed_local().compat()
}


// Verify close_code method.
//
#[ wasm_bindgen_test(async) ]
//
pub fn close_reason_invalid_code() -> impl Future01<Item = (), Error = JsValue>
{
	let _ = console_log::init_with_level( Level::Trace );

	info!( "starting test: close_reason_invalid_code" );

	async
	{
		let (ws, _wsio) = WsStream::connect( URL, None ).await.expect_throw( "Could not create websocket" );

		let res = ws.close_reason( 500, "Normal Shutdown" ).await;

		assert_eq!( &WsErrKind::InvalidCloseCode(500), res.unwrap_err().kind() );

		Ok(())

	}.boxed_local().compat()
}


// Verify close_code method.
//
#[ wasm_bindgen_test(async) ]
//
pub fn close_reason_invalid() -> impl Future01<Item = (), Error = JsValue>
{
	let _ = console_log::init_with_level( Level::Trace );

	info!( "starting test: close_reason_invalid" );

	async
	{
		let (ws, _wsio) = WsStream::connect( URL, None ).await.expect_throw( "Could not create websocket" );

		let res = ws.close_reason( 1000, vec![ "a"; 124 ].join( "" ) ).await;

		assert_eq!( &WsErrKind::ReasonStringToLong, res.unwrap_err().kind() );

		Ok(())

	}.boxed_local().compat()
}



// Verify Debug impl.
//
#[ wasm_bindgen_test(async) ]
//
pub fn debug() -> impl Future01<Item = (), Error = JsValue>
{
	let _ = console_log::init_with_level( Level::Trace );

	info!( "starting test: debug" );

	async
	{
		let (ws, _wsio) = WsStream::connect( URL, None ).await.expect_throw( "Could not create websocket" );

		assert_eq!( format!( "WsStream for connection: {}", URL ), format!( "{:?}", ws ) );

		Ok(())

	}.boxed_local().compat()
}

