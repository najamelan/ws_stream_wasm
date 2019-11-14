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
async fn connect_wrong_port()
{
	let _ = console_log::init_with_level( Level::Trace );

	info!( "starting test: connect_wrong_port" );

	let err = WsStream::connect( "ws://127.0.0.1:33212/", None ).await;

	assert!( err.is_err() );

	let err = err.unwrap_err();

	assert_eq!
	(
		&WsErrKind::ConnectionFailed
		{
			event: CloseEvent
			{
				was_clean: false,
				code     : 1006 ,
				reason   : "".to_string(),
			}
		},
		err.kind()
	);
}



// WsStream::connect: Verify error when connecting to a forbidden port
//
#[ wasm_bindgen_test(async) ]
//
async fn connect_forbidden_port()
{
	let _ = console_log::init_with_level( Level::Trace );

	info!( "starting test: connect_forbidden_port" );

	let err = WsStream::connect( "ws://127.0.0.1:6666/", None ).await;

	assert!( err.is_err() );

	let err = err.unwrap_err();

	assert_eq!( &WsErrKind::ForbiddenPort, err.kind() );
}



// WsStream::connect: Verify error when connecting to wss:// on ws:// server
//
#[ wasm_bindgen_test(async) ]
//
async fn connect_wrong_wss()
{
	let _ = console_log::init_with_level( Level::Trace );

	info!( "starting test: connect_wrong_wss" );

	let err = WsStream::connect( "wss://127.0.0.1:3212/", None ).await;

	assert!( err.is_err() );

	let err = err.unwrap_err();

	assert_eq!
	(
		&WsErrKind::ConnectionFailed
		{
			event: CloseEvent
			{
				was_clean: false,
				code     : 1006 ,
				reason   : "".to_string(),
			}
		},
		err.kind()
	);
}



// WsStream::connect: Verify error when connecting to a wrong scheme
//
#[ wasm_bindgen_test(async) ]
//
async fn connect_wrong_scheme()
{
	let _ = console_log::init_with_level( Level::Trace );

	info!( "starting test: connect_wrong_scheme" );

	let err = WsStream::connect( "http://127.0.0.1:3212/", None ).await;

	assert!( err.is_err() );

	let err = err.unwrap_err();

	assert_eq!( &WsErrKind::InvalidUrl{ supplied: "http://127.0.0.1:3212/".to_string() }, err.kind() );
}



// Verify state method.
//
#[ wasm_bindgen_test(async) ]
//
async fn state()
{
	let _ = console_log::init_with_level( Level::Trace );

	info!( "starting test: state" );

	let (ws, wsio) = WsStream::connect( URL, None ).await.expect_throw( "Could not create websocket" );

	assert_eq!( WsState::Open, ws  .ready_state() );
	assert_eq!( WsState::Open, wsio.ready_state() );

	ws.wrapped().close().expect_throw( "close WebSocket" );

	assert_eq!( WsState::Closing, ws  .ready_state() );
	assert_eq!( WsState::Closing, wsio.ready_state() );

	ws.close().await.expect_throw( "close ws" );

	assert_eq!( WsState::Closed, ws  .ready_state() );
	assert_eq!( WsState::Closed, wsio.ready_state() );
}


// Verify closing from WsIo.
//
#[ wasm_bindgen_test(async) ]
//
async fn close_from_wsio()
{
	let _ = console_log::init_with_level( Level::Trace );

	info!( "starting test: close_from_wsio" );

	let (ws, mut wsio) = WsStream::connect( URL, None ).await.expect_throw( "Could not create websocket" );

	assert_eq!( WsState::Open, ws.ready_state() );

	SinkExt::close( &mut wsio ).await.expect( "close wsio sink" );

	assert_eq!( WsState::Closed, wsio.ready_state() );
	assert_eq!( WsState::Closed, ws  .ready_state() );
}




// Verify url method.
//
#[ wasm_bindgen_test(async) ]
//
async fn url()
{
	let _ = console_log::init_with_level( Level::Trace );

	info!( "starting test: url" );

	let (ws, _wsio) = WsStream::connect( URL, None ).await.expect_throw( "Could not create websocket" );

	assert_eq!( URL, ws.url() );
}




// Verify protocols.
//
#[ wasm_bindgen_test(async) ]
//
async fn no_protocols()
{
	let _ = console_log::init_with_level( Level::Trace );

	info!( "starting test: no_protocols" );

	let (ws, _wsio) = WsStream::connect( URL, None ).await.expect_throw( "Could not create websocket" );

	assert_eq!( "", ws.protocol() );
}




/*
// Verify protocols.
// This doesn't work with tungstenite for the moment.
//
#[ wasm_bindgen_test(async) ]
//
async fn protocols_server_accept_none()
{
	let _ = console_log::init_with_level( Level::Trace );

	info!( "starting test: protocols_server_accept_none" );

	let (ws, _wsio) = WsStream::connect( URL, vec![ "chat" ] ).await.expect_throw( "Could not create websocket" );

	assert_eq!( "", ws.protocol() );
}
*/



// Verify close_code method.
//
#[ wasm_bindgen_test(async) ]
//
async fn close_twice()
{
	let _ = console_log::init_with_level( Level::Trace );

	info!( "starting test: close_twice" );

	let (ws, _wsio) = WsStream::connect( URL, None ).await.expect_throw( "Could not create websocket" );

	let res = ws.close().await;

	assert!( res.is_ok() );

	assert_eq!( ws.close       ()                         .await.unwrap_err().kind(), &WsErrKind::ConnectionNotOpen );
	assert_eq!( ws.close_code  ( 1000                    ).await.unwrap_err().kind(), &WsErrKind::ConnectionNotOpen );
	assert_eq!( ws.close_reason( 1000, "Normal shutdown" ).await.unwrap_err().kind(), &WsErrKind::ConnectionNotOpen );
}



#[ wasm_bindgen_test(async) ]
//
async fn close_code_valid()
{
	let _ = console_log::init_with_level( Level::Trace );

	info!( "starting test: close_code_valid" );

	let (ws, _wsio) = WsStream::connect( URL, None ).await.expect_throw( "Could not create websocket" );

	let res = ws.close_code( 1000 ).await;

	assert!( res.is_ok() );
}


// Verify close_code method.
//
#[ wasm_bindgen_test(async) ]
//
async fn close_code_invalid()
{
	let _ = console_log::init_with_level( Level::Trace );

	info!( "starting test: close_code_invalid" );

	let (ws, _wsio) = WsStream::connect( URL, None ).await.expect_throw( "Could not create websocket" );

	let res = ws.close_code( 500 ).await;

	assert_eq!( &WsErrKind::InvalidCloseCode{ supplied: 500 }, res.unwrap_err().kind() );
}


// Verify close_code method.
//
#[ wasm_bindgen_test(async) ]
//
async fn close_reason_valid()
{
	let _ = console_log::init_with_level( Level::Trace );

	info!( "starting test: close_reason_valid" );

	let (ws, _wsio) = WsStream::connect( URL, None ).await.expect_throw( "Could not create websocket" );

	let res = ws.close_reason( 1000, "Normal shutdown" ).await;

	assert!( res.is_ok() );
}


// Verify close_code method.
//
#[ wasm_bindgen_test(async) ]
//
async fn close_reason_invalid_code()
{
	let _ = console_log::init_with_level( Level::Trace );

	info!( "starting test: close_reason_invalid_code" );

	let (ws, _wsio) = WsStream::connect( URL, None ).await.expect_throw( "Could not create websocket" );

	let res = ws.close_reason( 500, "Normal Shutdown" ).await;

	assert_eq!( &WsErrKind::InvalidCloseCode{ supplied: 500 }, res.unwrap_err().kind() );
}


// Verify close_code method.
//
#[ wasm_bindgen_test(async) ]
//
async fn close_reason_invalid()
{
	let _ = console_log::init_with_level( Level::Trace );

	info!( "starting test: close_reason_invalid" );

	let (ws, _wsio) = WsStream::connect( URL, None ).await.expect_throw( "Could not create websocket" );

	let res = ws.close_reason( 1000, vec![ "a"; 124 ].join( "" ) ).await;

	assert_eq!( &WsErrKind::ReasonStringToLong, res.unwrap_err().kind() );
}



// Verify Debug impl.
//
#[ wasm_bindgen_test(async) ]
//
async fn debug()
{
	let _ = console_log::init_with_level( Level::Trace );

	info!( "starting test: debug" );

	let (ws, _wsio) = WsStream::connect( URL, None ).await.expect_throw( "Could not create websocket" );

	assert_eq!( format!( "WsStream for connection: {}", URL ), format!( "{:?}", ws ) );

	ws.close().await.expect_throw( "close" );
}

