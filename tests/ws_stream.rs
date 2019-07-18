#![ feature( async_await, trait_alias )]
wasm_bindgen_test_configure!(run_in_browser);


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

		ws.close().await;

		assert_eq!( WsState::Closed, ws  .ready_state() );
		assert_eq!( WsState::Closed, wsio.ready_state() );

		Ok(())

	}.boxed_local().compat()
}


// Verify state method.
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


// Verify close_code method.
//
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
