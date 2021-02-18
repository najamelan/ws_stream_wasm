wasm_bindgen_test_configure!(run_in_browser);

// What's tested:
//
// Tests send to an echo server which just bounces back all data.
//
// ✔ Verify close events are emitted.
//
use
{
	futures::prelude      :: { *                         } ,
	wasm_bindgen::prelude :: { *                         } ,
	wasm_bindgen_test     :: { *                         } ,
	log                   :: { *                         } ,
	ws_stream_wasm        :: { *                         } ,
	pharos                :: { ObserveConfig, Observable } ,
	// web_sys               :: { console::log_1 as dbg } ,
};


const URL: &str = "ws://127.0.0.1:3212";



// Verify close events are emitted.
//
#[ wasm_bindgen_test ]
//
async fn close_events()
{
	let _ = console_log::init_with_level( Level::Trace );

	info!( "starting test: close_events" );

	let (mut ws, _wsio) = WsMeta::connect( URL, None ).await.expect_throw( "Could not create websocket" );

	let mut evts = ws.observe( ObserveConfig::default() ).await.expect( "observe" );

	ws.close().await.expect_throw( "close ws" );

	assert!( evts.next().await.unwrap_throw().is_closing() );
	assert!( evts.next().await.unwrap_throw().is_closed()  );
}

