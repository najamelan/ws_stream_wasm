wasm_bindgen_test_configure!(run_in_browser);


// What's tested:
//
// Tests send to an echo server which just bounces back all data.
//
// ✔ Frame with a BytesCodec and verify that a round trip returns identical data
// ✔ Send 1MB data in a custom struct serialized with cbor_codec
//
use
{
	wasm_bindgen::prelude :: { *                                    } ,
	wasm_bindgen_test     :: { *                                    } ,
	ws_stream_wasm        :: { *                                    } ,
	log                   :: { *                                    } ,
	rand_xoshiro          :: { *                                    } ,
	rand                  :: { RngCore, SeedableRng                 } ,
	tokio                 :: { codec::{ BytesCodec, Decoder }       } ,
	bytes                 :: { Bytes                                } ,
	futures               :: { stream::{ StreamExt }, sink::SinkExt } ,
	futures               :: { AsyncReadExt, compat::Compat         } ,
	futures::compat       :: { Stream01CompatExt, Sink01CompatExt   } ,
	tokio::prelude        :: { Stream                               } ,
	serde                 :: { Serialize, Deserialize               } ,
	tokio_serde_cbor      :: { Codec                                } ,
	// web_sys               :: { console::log_1 as dbg                               } ,
};



const URL: &str = "ws://127.0.0.1:3212";



async fn connect() -> (WsStream, Compat<WsIo>)
{
	let (ws, wsio) = WsStream::connect( URL, None ).await.expect_throw( "Could not create websocket" );

	( ws, AsyncReadExt::compat(wsio) )
}



// Verify that a round trip to an echo server generates identical data.
//
#[ wasm_bindgen_test( async ) ]
//
async fn data_integrity()
{
	// It's normal for this to fail, since different tests run in the same module and we can only
	// set the logger once. Since we don't know which test runs first, we ignore the Result.
	// Logging will only show up in the browser, not in wasm-pack test --headless.
	//
	let _ = console_log::init_with_level( Level::Trace );

	info!( "starting test: data_integrity" );

	let big_size   = 10240;
	let mut random = vec![ 0u8; big_size ];
	let mut rng    = Xoshiro256Plus::from_seed( [ 1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0 ] );

	rng.fill_bytes( &mut random );

	let dataset: Vec<(&str, usize, Bytes)> = vec!
	[
		( "basic", 18, Bytes::from_static( b"Hello from browser" ) ),

		// 20 random bytes, not valid unicode
		//
		( "random bytes", 20, Bytes::from( vec![ 72, 31, 238, 236, 85, 240, 197, 235, 149, 238, 245, 206, 227, 201, 139, 63, 173, 214, 158, 134 ] ) ),

		// Test with something big:
		//
		( "big random", big_size, Bytes::from( random ) ),
	];

	for data in dataset
	{
		echo( data.0, data.1, data.2 ).await;
	}
}



// Send data to an echo server and verify that what returns is exactly the same
//
async fn echo( name: &str, size: usize, data: Bytes )
{
	info!( "   Enter echo: {}", name );

	let (_ws, wsio) = connect().await;
	let (tx, rx)    = BytesCodec::new().framed( wsio ).split();

	let mut tx = tx.sink_compat();
	let mut rx = rx.compat();

	tx.send( data.clone() ).await.expect_throw( "Failed to write to websocket" );

	let mut result: Vec<u8> = Vec::new();

	while result.len() < size
	{
		let msg        = rx.next().await.expect_throw( "read message" );
		let buf: &[u8] = msg.as_ref().expect_throw( "msg.as_ref()" );
		result.extend_from_slice( buf );
	}

	assert_eq!( &data, &Bytes::from( result  ) );

	tx.close().await.expect_throw( "close" );
}





/////////////////////
// With serde-cbor //
/////////////////////


#[ derive( Debug, Clone, Serialize, Deserialize, PartialEq, Eq ) ]
//
struct Data
{
	hello: String   ,
	data : Vec<u32> ,
	num  : u64      ,
}


// Verify that a round trip to an echo server generates identical data. This test includes a big (1MB)
// piece of data.
//
#[ wasm_bindgen_test( async ) ]
//
async fn data_integrity_cbor()
{
	// It's normal for this to fail, since different tests run in the same module and we can only
	// set the logger once. Since we don't know which test runs first, we ignore the Result.
	// Logging will only show up in the browser, not in wasm-pack test --headless.
	//
	let _ = console_log::init_with_level( Level::Trace );

	info!( "starting test: data_integrity_cbor" );

	let dataset: Vec<Data> = vec!
	[
		Data{ hello: "Hello CBOR - basic".to_string(), data: vec![ 0, 33245, 3, 36 ], num: 3948594 },

		// Test with something big
		//
		Data{ hello: "Hello CBOR - 1MB data".to_string(), data: vec![ 1; 1_024_000 ], num: 3948595 },
	];

	for data in dataset
	{
		echo_cbor( data ).await;
	}
}


// Send data to an echo server and verify that what returns is exactly the same
//
async fn echo_cbor( data: Data )
{
	info!( "   Enter echo_cbor: {}", &data.hello );

	let (_ws, wsio) = connect().await;

	let codec: Codec<Data, Data>  = Codec::new().packed( true );
	let (tx, rx) = codec.framed( wsio ).split();

	let mut tx = tx .sink_compat();
	let mut rx = rx .compat();


	tx.send( data.clone() ).await.expect_throw( "Failed to write to websocket" );

	let msg = rx.next().await.expect_throw( "read message" ).expect_throw( "msg" );

	assert_eq!( data, msg );

	tx.close().await.expect_throw( "close" );
}



