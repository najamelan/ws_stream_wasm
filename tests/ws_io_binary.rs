
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


const URL: &str = "ws://127.0.0.1:3212/";


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
		let (_ws, wsbio) = WsStream::connect_binary( URL, None ).await.expect_throw( "Could not create websocket" );

		let (mut tx, mut rx) = wsbio.split();
		let message          = b"Hello from browser".to_vec();


		tx.send( message.clone() ).await

			.expect_throw( "Failed to write to websocket" );


		let msg    = rx.next().await;
		let result = msg.expect_throw( "Stream closed" ).expect_throw( "no io error" );

		assert_eq!( message, result );

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
		let (ws, mut wsbio) = WsStream::connect_binary( URL, None ).await.expect_throw( "Could not create websocket" );

		ws.wrapped().close().expect_throw( "close connection" );

		let res = wsbio.send( b"Hello from browser".to_vec() ).await;

		assert_eq!( &WsErrKind::ConnectionClosed, res.unwrap_err().kind() );

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
		let (ws, mut wsbio) = WsStream::connect_binary( URL, None ).await.expect_throw( "Could not create websocket" );

		ws.close().await;

		let res = wsbio.send( b"Hello from browser".to_vec() ).await;

		assert_eq!( &WsErrKind::ConnectionClosed, res.unwrap_err().kind() );

		Ok(())

	}.boxed_local().compat()
}




// Verify Display impl.
//
#[ wasm_bindgen_test(async) ]
//
pub fn display() -> impl Future01<Item = (), Error = JsValue>
{
	let _ = console_log::init_with_level( Level::Trace );

	info!( "starting test: display" );

	async
	{
		let (_ws, wsbio) = WsStream::connect_binary( URL, None ).await.expect_throw( "Could not create websocket" );

		assert_eq!( format!( "WsIoBinary for connection: {}", URL ), format!( "{}", wsbio ) );

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
		let (_ws, wsbio) = WsStream::connect_binary( URL, None ).await.expect_throw( "Could not create websocket" );

		assert_eq!( format!( "WsIoBinary for connection: {}", URL ), format!( "{:?}", wsbio ) );

		Ok(())

	}.boxed_local().compat()
}
