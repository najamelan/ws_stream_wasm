#![ feature( async_await, trait_alias )]
wasm_bindgen_test_configure!(run_in_browser);

// What's tested:
//
// Tests send to an echo server which just bounces back all data.
//
// ✔ Verify close events are emitted.
//
use
{
	futures_01            :: { Future as Future01 } ,
	futures::prelude      :: { *                  } ,
	wasm_bindgen::prelude :: { *                  } ,
	wasm_bindgen_test     :: { *                  } ,
	log                   :: { *                  } ,
	ws_stream_wasm        :: { * } ,
	pharos                :: { UnboundedObservable } ,
	// web_sys               :: { console::log_1 as dbg               } ,
};


const URL: &str = "ws://127.0.0.1:3212";



// Verify close events are emitted.
//
#[ wasm_bindgen_test(async) ]
//
pub fn close_events() -> impl Future01<Item = (), Error = JsValue>
{
	let _ = console_log::init_with_level( Level::Trace );

	info!( "starting test: close_events" );

	async
	{
		let (mut ws, _wsio) = WsStream::connect( URL, None ).await.expect_throw( "Could not create websocket" );

		let mut evts = ws.observe_unbounded();

		ws.close().await.expect_throw( "close ws" );

		assert_eq!( WsEventType::CLOSING, evts.next().await.unwrap_throw().ws_type() );
		assert_eq!( WsEventType::CLOSE  , evts.next().await.unwrap_throw().ws_type() );

		Ok(())

	}.boxed_local().compat()
}

