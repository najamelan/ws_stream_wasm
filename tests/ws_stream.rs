#![ feature( async_await, trait_alias )]
wasm_bindgen_test_configure!(run_in_browser);


use
{
	futures_01            :: Future as Future01,
	futures::prelude      :: * ,
	wasm_bindgen::prelude :: * ,
	wasm_bindgen_test     :: * ,
	ws_stream_wasm        :: * ,
	log                   :: * ,
};


const URL_WSSTREAM: &str = "ws://127.0.0.1:3212/";
const URL_WS      : &str = "ws://127.0.0.1:3312/";




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
		let (ws, wsio) = WsStream::connect( URL_WS, None ).await.expect_throw( "Could not create websocket" );

		assert_eq!( WsState::Open, ws  .ready_state() );
		assert_eq!( WsState::Open, wsio.ready_state() );

		ws.close().await;

		assert_eq!( WsState::Closed, ws  .ready_state() );
		assert_eq!( WsState::Closed, wsio.ready_state() );

		let r: Result<(), wasm_bindgen::JsValue> = Ok(());

		r

	}.boxed_local().compat()
}


// Verify state method.
//
#[ wasm_bindgen_test(async) ]
//
pub fn close_from_wsio() -> impl Future01<Item = (), Error = JsValue>
{
	let _ = console_log::init_with_level( Level::Trace );

	info!( "starting test: state" );

	async
	{
		let (ws, mut wsio) = WsStream::connect( URL_WS, None ).await.expect_throw( "Could not create websocket" );

		assert_eq!( WsState::Open, ws.ready_state() );

		wsio.close().await.expect( "close wsio sink" );

		assert_eq!( WsState::Closed, wsio.ready_state() );
		assert_eq!( WsState::Closed, ws  .ready_state() );

		let r: Result<(), wasm_bindgen::JsValue> = Ok(());

		r

	}.boxed_local().compat()
}

