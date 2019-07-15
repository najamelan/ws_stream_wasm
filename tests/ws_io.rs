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



// Verify that a round trip to an echo server generates identical textual data.
//
#[ wasm_bindgen_test(async) ]
//
pub fn round_trip_text() -> impl Future01<Item = (), Error = JsValue>
{
	let _ = console_log::init_with_level( Level::Trace );

	info!( "starting test: round_trip" );

	async
	{
		let (_ws, wsio) = WsStream::connect( URL_WS ).await.expect_throw( "Could not create websocket" );


		let (mut tx, mut rx) = wsio.split();
		let message          = "Hello from browser".to_string();


		tx.send( JsMsgEvtData::Text( message.clone() ) ).await

			.expect_throw( "Failed to write to websocket" );


		let msg    = rx.next().await;
		let result = &msg.expect_throw( "Stream closed" );

		assert_eq!( JsMsgEvtData::Text( message ), result.data() );

		let r: Result<(), wasm_bindgen::JsValue> = Ok(());

		r

	}.boxed_local().compat()
}



// Verify that a round trip to an echo server generates identical binary data.
//
#[ wasm_bindgen_test(async) ]
//
pub fn round_trip_binary() -> impl Future01<Item = (), Error = JsValue>
{
	let _ = console_log::init_with_level( Level::Trace );

	info!( "starting test: round_trip" );

	async
	{
		let (_ws, wsio) = WsStream::connect( URL_WSSTREAM ).await.expect_throw( "Could not create websocket" );

		let (mut tx, mut rx) = wsio.split();
		let message          = b"Hello from browser".to_vec();


		tx.send( JsMsgEvtData::Binary( message.clone() ) ).await

			.expect_throw( "Failed to write to websocket" );


		let msg    = rx.next().await;
		let result = &msg.unwrap();

		assert_eq!( JsMsgEvtData::Binary( message ), result.data() );

		let r: Result<(), wasm_bindgen::JsValue> = Ok(());

		r

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
		let (ws, _wsio) = WsStream::connect( URL_WSSTREAM ).await.expect_throw( "Could not create websocket" );

		assert_eq!( URL_WSSTREAM, ws.url() );

		let r: Result<(), wasm_bindgen::JsValue> = Ok(());

		r

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
		let (ws, _wsio) = WsStream::connect( URL_WSSTREAM ).await.expect_throw( "Could not create websocket" );

		assert_eq!( WsState::OPEN, ws.ready_state() );

		ws.close().await;

		assert_eq!( WsState::CLOSED, ws.ready_state() );

		let r: Result<(), wasm_bindgen::JsValue> = Ok(());

		r

	}.boxed_local().compat()
}

