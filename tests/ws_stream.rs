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
	futures::prelude      :: * ,
	log                   :: * ,
	pharos                :: * ,
	std::marker           :: PhantomData ,
	wasm_bindgen::prelude :: * ,
	wasm_bindgen_futures  :: spawn_local ,
	wasm_bindgen_test     :: * ,
	ws_stream_wasm        :: * ,
};



const URL   : &str = "ws://127.0.0.1:3212/";
const URL_TT: &str = "ws://127.0.0.1:3312/";



// Verify that both WsStream and WsMeta are Send for now. The browser API's are not Send,
// and this is not meant to be send accross threads. However some API's need to require
// Send (eg async that can be spawned on a thread pool). However on wasm you can spawn them
// on a single threaded executor and they shouldn't require Send. On the long term it would
// be nice to have a better solution.
//
#[ wasm_bindgen_test ]
//
fn sendness()
{
	struct SendNess<T: Send + Sync>{ _phantom: PhantomData<T> }

	let _x = SendNess::<WsStream>{ _phantom: PhantomData };
	let _x = SendNess::<WsMeta  >{ _phantom: PhantomData };
}



// Verify that a round trip to an echo server generates identical textual data.
//
#[ wasm_bindgen_test ]
//
async fn round_trip_text()
{
	let _ = console_log::init_with_level( Level::Trace );

	info!( "starting test: round_trip_text" );

	let (_ws, mut wsio) = WsMeta::connect( URL_TT, None ).await.expect_throw( "Could not create websocket" );
	let message         = "Hello from browser".to_string();


	wsio.send( WsMessage::Text( message.clone() ) ).await

		.expect_throw( "Failed to write to websocket" );


	let msg    = wsio.next().await;
	let result = msg.expect_throw( "Stream closed" );

	assert_eq!( WsMessage::Text( message ), result );
}



// Verify that a round trip to an echo server generates identical binary data.
//
#[ wasm_bindgen_test ]
//
async fn round_trip_binary()
{
	let _ = console_log::init_with_level( Level::Trace );

	info!( "starting test: round_trip_binary" );

	let (_ws, mut wsio) = WsMeta::connect( URL, None ).await.expect_throw( "Could not create websocket" );
	let message         = b"Hello from browser".to_vec();


	wsio.send( WsMessage::Binary( message.clone() ) ).await

		.expect_throw( "Failed to write to websocket" );


	let msg    = wsio.next().await;
	let result = msg.expect_throw( "Stream closed" );

	assert_eq!( WsMessage::Binary( message ), result );
}



#[ wasm_bindgen_test ]
//
async fn send_while_closing()
{
	info!( "starting test: send_while_closing" );

	let (ws, mut wsio) = WsMeta::connect( URL, None ).await.expect_throw( "Could not create websocket" );

	ws.wrapped().close().expect_throw( "close connection" );

	let res = wsio.send( WsMessage::Text("Hello from browser".into() ) ).await;

	assert_eq!( WsErr::ConnectionNotOpen, res.unwrap_err() );
}



#[ wasm_bindgen_test ]
//
async fn send_after_close()
{
	info!( "starting test: send_after_close" );

	let (ws, mut wsio) = WsMeta::connect( URL, None ).await.expect_throw( "Could not create websocket" );

	ws.close().await.expect_throw( "close ws" );

	let res = wsio.send( WsMessage::Text("Hello from browser".into() ) ).await;

	assert_eq!( WsErr::ConnectionNotOpen, res.unwrap_err() );
}



// Verify closing that when closing from WsMeta, WsStream next() returns none.
//
#[ wasm_bindgen_test ]
//
async fn close_from_wsstream()
{
	let _ = console_log::init_with_level( Level::Trace );

	info!( "starting test: close_from_wsstream" );

	let (ws, mut wsio) = WsMeta::connect( URL, None ).await.expect_throw( "Could not create websocket" );

	ws.close().await.expect_throw( "close ws" );

	assert!( wsio.next().await.is_none() );
}



// Verify that closing wakes up a task pending on poll_next()
//
#[ wasm_bindgen_test ]
//
async fn close_from_wsstream_while_pending()
{
	let _ = console_log::init_with_level( Level::Trace );

	info!( "starting test: close_from_wsstream_while_pending" );

	let (ws, mut wsio) = WsMeta::connect( URL, None ).await.expect_throw( "Could not create websocket" );

	spawn_local( async move { ws.close().await.expect_throw( "close ws" ); } );

	// if we don't wake up the task, this will hang
	//
	assert!( wsio.next().await.is_none() );
}



// Verify that closing wakes up a task pending on poll_next()
//
#[ wasm_bindgen_test ]
//
async fn close_event_from_sink()
{
	let _ = console_log::init_with_level( Level::Trace );

	info!( "starting test: close_event_from_sink" );

	let (mut ws, mut wsio) = WsMeta::connect( URL, None ).await.expect_throw( "Could not create websocket" );

	let mut evts = ws.observe( ObserveConfig::default() ).await.expect( "observe" );

	wsio.close().await.expect_throw( "close ws" );

	assert!( evts.next().await.unwrap_throw().is_closing() );
	assert!( evts.next().await.unwrap_throw().is_closed()  );
}



// Verify that closing wakes up a task pending on poll_next()
//
#[ wasm_bindgen_test ]
//
async fn close_event_from_async_write()
{
	let _ = console_log::init_with_level( Level::Trace );

	info!( "starting test: close_event_from_async_write" );

	let (mut ws, stream) = WsMeta::connect( URL, None ).await.expect_throw( "Could not create websocket" );
	let mut stream = stream.into_io();

	let mut evts = ws.observe( ObserveConfig::default() ).await.expect( "observe" );

	stream.close().await.expect_throw( "close ws" );

	assert!( evts.next().await.unwrap_throw().is_closing() );
	assert!( evts.next().await.unwrap_throw().is_closed()  );
}



// Verify Debug impl.
//
#[ wasm_bindgen_test ]
//
async fn debug()
{
	let _ = console_log::init_with_level( Level::Trace );

	info!( "starting test: debug" );

	let (_ws, mut wsio) = WsMeta::connect( URL, None ).await.expect_throw( "Could not create websocket" );

	assert_eq!( format!( "WsStream for connection: {}", URL ), format!( "{:?}", wsio ) );

	wsio.close().await.expect_throw( "close" );
}
