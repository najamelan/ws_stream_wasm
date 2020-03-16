wasm_bindgen_test_configure!(run_in_browser);

// What's tested:
//
// Tests send to an echo server which just bounces back all data.
//
// ✔ Frame with a BytesCodec and verify that a round trip returns identical data
// ✔ Use a LinesCodec and get back identical lines
//
use
{
	wasm_bindgen::prelude :: { *                                    } ,
	wasm_bindgen_test     :: { *                                    } ,
	ws_stream_wasm        :: { *                                    } ,
	log                   :: { *                                    } ,
	rand_xoshiro          :: { *                                    } ,
	rand                  :: { RngCore, SeedableRng                 } ,
	bytes                 :: { Bytes                                } ,
	futures               :: { stream::StreamExt, sink::SinkExt,    } ,
	futures_codec         :: { Framed, LinesCodec, BytesCodec       } ,
	serde                 :: { Serialize, Deserialize               } ,
	// web_sys               :: { console::log_1 as dbg               } ,
	async_io_stream       :: { IoStream                                 } ,
};


const URL: &str = "ws://127.0.0.1:3212";



async fn connect() -> (WsMeta, IoStream<WsStreamIo, Vec<u8>>)
{
	let (ws, wsio) = WsMeta::connect( URL, None ).await.expect_throw( "Could not create websocket" );

	(ws, wsio.into_io())
}



// Verify that a round trip to an echo server generates identical data.
//
#[ wasm_bindgen_test( async ) ]
//
async fn data_integrity()
{
	let _ = console_log::init_with_level( Level::Trace );

	info!( "starting test: data_integrity" );

	let big_size   = 10240;
	let mut random = vec![ 0; big_size ];
	let mut rng    = Xoshiro256Plus::from_seed( [ 1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0 ] );

	rng.fill_bytes( &mut random );

	let dataset: Vec<(&str, usize, Bytes)> = vec!
	[
		( "basic"       , 18, Bytes::from_static( b"Hello from browser" ) ),

		// 20 random bytes, not valid unicode
		//
		( "random bytes", 20, Bytes::from( vec![ 72, 31, 238, 236, 85, 240, 197, 235, 149, 238, 245, 206, 227, 201, 139, 63, 173, 214, 158, 134 ] ) ),

		// Test with something big:
		//
		( "big random"  , big_size, Bytes::from( random ) ),
	];

	for data in dataset
	{
		echo( data.0, data.1, data.2 ).await;
	}
}



// Send data to an echo server and verify that what returns is exactly the same
// We run 2 connections in parallel, the second one we verify that we can use a reference
// to a WsMeta
//
async fn echo( name: &str, size: usize, data: Bytes )
{
	info!( "   Enter echo: {}", name );

	let (_ws , wsio) = connect().await;

	let mut framed = Framed::new( wsio, BytesCodec {} );

	framed.send( data.clone() ).await.expect_throw( "Failed to write to websocket" );

	let mut result: Vec<u8> = Vec::new();

	while &result.len() < &size
	{
		let msg = framed.next().await.expect_throw( "Some" ).expect_throw( "Receive bytes" );
		let buf: &[u8] = msg.as_ref();
		result.extend( buf );
	}

	assert_eq!( &data, &Bytes::from( result  ) );

	framed.close().await.expect_throw( "close" );
}





/////////////////////
// With LinesCodec //
/////////////////////


#[ derive( Debug, Clone, Serialize, Deserialize, PartialEq, Eq ) ]
//
struct Data
{
	hello: String   ,
	data : Vec<u32> ,
	num  : u64      ,
}


// Verify that a round trip to an echo server generates identical data.
//
#[ wasm_bindgen_test( async ) ]
//
async fn lines_integrity()
{
	let _ = console_log::init_with_level( Level::Trace );

	info!( "starting test: lines_integrity" );


	let (_ws , wsio ) = connect().await;
	let mut framed = Framed::new( wsio, LinesCodec {} );

	info!( "lines_integrity: start sending" );

	framed.send( "A line\n"       .to_string() ).await.expect_throw( "Send a line"        );
	framed.send( "A second line\n".to_string() ).await.expect_throw( "Send a second line" );
	framed.send( "A third line\n" .to_string() ).await.expect_throw( "Send a third line"  );

	info!( "lines_integrity: start receiving" );

	let one   = framed.next().await.expect_throw( "Some" ).expect_throw( "Receive a line"        );
	let two   = framed.next().await.expect_throw( "Some" ).expect_throw( "Receive a second line" );
	let three = framed.next().await.expect_throw( "Some" ).expect_throw( "Receive a third line"  );

	info!( "lines_integrity: start asserting" );

	assert_eq!( "A line\n"       , &one   );
	assert_eq!( "A second line\n", &two   );
	assert_eq!( "A third line\n" , &three );

	info!( "lines_integrity: done" );

	framed.close().await.expect_throw( "close" );
}




