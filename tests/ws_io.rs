#![ feature( trait_alias )]
wasm_bindgen_test_configure!(run_in_browser);



// What's tested:
//
// Tests send to an echo server which just bounces back all data.
//
// ✔ Send a WsMessage::Text   and verify we get an identical WsMessage back.
// ✔ Send a WsMessage::Binary and verify we get an identical WsMessage back.
// ✔ Send while closing and verify the error
// ✔ Send while closed  and verify the error
// ✔ Test Debug impl
//
// Note that AsyncRead/AsyncWrite are tested by futures_codec.rs and tokio_codec.rs
//
use
{
	futures_01            :: Future as Future01       ,
	wasm_bindgen_futures  :: futures_0_3::spawn_local ,
	futures::prelude      :: * ,
	futures::sink         :: * ,
	futures::io           :: * ,
	wasm_bindgen::prelude :: * ,
	wasm_bindgen_test     :: * ,
	ws_stream_wasm        :: * ,
	log                   :: * ,
	pharos                :: * ,
};



const URL   : &str = "ws://127.0.0.1:3212/";
const URL_TT: &str = "ws://127.0.0.1:3312/";



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
		let (_ws, mut wsio) = WsStream::connect( URL_TT, None ).await.expect_throw( "Could not create websocket" );
		let message         = "Hello from browser".to_string();


		wsio.send( WsMessage::Text( message.clone() ) ).await

			.expect_throw( "Failed to write to websocket" );


		let msg    = wsio.next().await;
		let result = msg.expect_throw( "Stream closed" );

		assert_eq!( WsMessage::Text( message ), result );

		Ok(())

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
		let (_ws, mut wsio) = WsStream::connect( URL, None ).await.expect_throw( "Could not create websocket" );
		let message         = b"Hello from browser".to_vec();


		wsio.send( WsMessage::Binary( message.clone() ) ).await

			.expect_throw( "Failed to write to websocket" );


		let msg    = wsio.next().await;
		let result = msg.expect_throw( "Stream closed" );

		assert_eq!( WsMessage::Binary( message ), result );

		Ok(())

	}.boxed_local().compat()
}



#[ wasm_bindgen_test(async) ]
//
fn send_while_closing() -> impl Future01<Item = (), Error = JsValue>
{
	info!( "starting test: send_while_closing" );

	async
	{
		let (ws, mut wsio) = WsStream::connect( URL, None ).await.expect_throw( "Could not create websocket" );

		ws.wrapped().close().expect_throw( "close connection" );

		let res = wsio.send( WsMessage::Text("Hello from browser".into() ) ).await;

		assert_eq!( &WsErrKind::ConnectionNotOpen, res.unwrap_err().kind() );

		Ok(())

	}.boxed_local().compat()
}



#[ wasm_bindgen_test(async) ]
//
fn send_after_close() -> impl Future01<Item = (), Error = JsValue>
{
	info!( "starting test: send_after_close" );

	async
	{
		let (ws, mut wsio) = WsStream::connect( URL, None ).await.expect_throw( "Could not create websocket" );

		ws.close().await.expect_throw( "close ws" );

		let res = wsio.send( WsMessage::Text("Hello from browser".into() ) ).await;

		assert_eq!( &WsErrKind::ConnectionNotOpen, res.unwrap_err().kind() );

		Ok(())

	}.boxed_local().compat()
}



// Verify closing that when closing from WsStream, WsIo next() returns none.
//
#[ wasm_bindgen_test(async) ]
//
pub fn close_from_wsstream() -> impl Future01<Item = (), Error = JsValue>
{
	let _ = console_log::init_with_level( Level::Trace );

	info!( "starting test: close_from_wsstream" );

	async
	{
		let (ws, mut wsio) = WsStream::connect( URL, None ).await.expect_throw( "Could not create websocket" );

		ws.close().await.expect_throw( "close ws" );

		assert!( wsio.next().await.is_none() );

		Ok(())

	}.boxed_local().compat()
}



// Verify that closing wakes up a task pending on poll_next()
//
#[ wasm_bindgen_test(async) ]
//
pub fn close_from_wsstream_while_pending() -> impl Future01<Item = (), Error = JsValue>
{
	let _ = console_log::init_with_level( Level::Trace );

	info!( "starting test: close_from_wsstream_while_pending" );

	async
	{
		let (ws, mut wsio) = WsStream::connect( URL, None ).await.expect_throw( "Could not create websocket" );

		spawn_local( async move { ws.close().await.expect_throw( "close ws" ); } );

		// if we don't wake up the task, this will hang
		//
		assert!( wsio.next().await.is_none() );

		Ok(())

	}.boxed_local().compat()
}



// Verify that closing wakes up a task pending on poll_next()
//
#[ wasm_bindgen_test(async) ]
//
pub fn close_event_from_sink() -> impl Future01<Item = (), Error = JsValue>
{
	let _ = console_log::init_with_level( Level::Trace );

	info!( "starting test: close_event_from_sink" );

	async
	{
		let (mut ws, mut wsio) = WsStream::connect( URL, None ).await.expect_throw( "Could not create websocket" );

		let mut evts = ws.observe_unbounded();

		SinkExt::close( &mut wsio ).await.expect_throw( "close ws" );

		assert_eq!( WsEventType::CLOSING, evts.next().await.unwrap_throw().ws_type() );
		assert_eq!( WsEventType::CLOSE  , evts.next().await.unwrap_throw().ws_type() );

		Ok(())

	}.boxed_local().compat()
}



// Verify that closing wakes up a task pending on poll_next()
//
#[ wasm_bindgen_test(async) ]
//
pub fn close_event_from_async_write() -> impl Future01<Item = (), Error = JsValue>
{
	let _ = console_log::init_with_level( Level::Trace );

	info!( "starting test: close_event_from_async_write" );

	async
	{
		let (mut ws, mut wsio) = WsStream::connect( URL, None ).await.expect_throw( "Could not create websocket" );

		let mut evts = ws.observe_unbounded();

		AsyncWriteExt::close( &mut wsio ).await.expect_throw( "close ws" );

		assert_eq!( WsEventType::CLOSING, evts.next().await.unwrap_throw().ws_type() );
		assert_eq!( WsEventType::CLOSE  , evts.next().await.unwrap_throw().ws_type() );

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
		let (_ws, mut wsio) = WsStream::connect( URL, None ).await.expect_throw( "Could not create websocket" );

		assert_eq!( format!( "WsIo for connection: {}", URL ), format!( "{:?}", wsio ) );

		SinkExt::close( &mut wsio ).await.expect_throw( "close" );

		Ok(())

	}.boxed_local().compat()
}
