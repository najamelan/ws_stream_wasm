#![ feature( async_await, trait_alias )]
wasm_bindgen_test_configure!(run_in_browser);



use
{
	wasm_bindgen::prelude :: { *                                                   } ,
	wasm_bindgen_test     :: { *                                                   } ,
	ws_stream_wasm        :: { *                                                   } ,
	log                   :: { *                                                   } ,
	rand_xoshiro          :: { *                                                   } ,
	rand                  :: { RngCore, SeedableRng                                } ,
	tokio                 :: { codec::{ BytesCodec, Decoder }                      } ,
	bytes                 :: { Bytes                                               } ,
	futures_01            :: { Future as Future01                                  } ,
	futures               :: { future::{ FutureExt, TryFutureExt }                 } ,
	futures               :: { stream::{ StreamExt, IntoAsyncRead }, sink::SinkExt } ,
	futures               :: { TryStreamExt, AsyncReadExt, compat::Compat          } ,
	futures::compat       :: { Stream01CompatExt, Sink01CompatExt                  } ,
	tokio::prelude        :: { Stream                                              } ,
	serde                 :: { Serialize, Deserialize                              } ,
	tokio_serde_cbor      :: { Codec                                               } ,
	// web_sys               :: { console::log_1 as dbg                               } ,
};



const URL: &str = "ws://127.0.0.1:3212";




async fn connect() -> (WsStream, Compat<IntoAsyncRead<WsIoBinary>>)
{
	let (ws, wsio) = WsStream::connect_binary( URL ).await.expect_throw( "Could not create websocket" );

	(ws, wsio.into_async_read().compat())
}



// Verify that a round trip to an echo server generates identical data.
//
#[ wasm_bindgen_test( async ) ]
//
pub fn data_integrity() -> impl Future01<Item = (), Error = JsValue>
{
	let _ = console_log::init_with_level( Level::Trace );

	console_log!( "starting test: data_integrity" );

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

	async move
	{
		for data in dataset
		{
			echo( data.0, data.1, data.2 ).await;
		}

		let r: Result<(), wasm_bindgen::JsValue> = Ok(());

		r

	}.boxed_local().compat()
}



// Send data to an echo server and verify that what returns is exactly the same
//
async fn echo( name: &str, size: usize, data: Bytes )
{
	console_log!( "   Enter echo: {}", name );

	let (_ws , wsio) = connect().await;
	let ( tx , rx  ) = BytesCodec::new().framed(  wsio  ).split();

	let mut tx = tx.sink_compat();
	let mut rx = rx.compat();

	tx.send( data.clone() ).await.expect_throw( "Failed to write to websocket" );

	let mut result: Vec<u8> = Vec::new();

	while result.len() < size
	{
		let msg = rx.next().await.unwrap_throw();
		let buf: &[u8] = msg.as_ref().unwrap_throw();
		result.extend( buf );
	}

	assert_eq!( &data, &Bytes::from( result  ) );
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
pub fn data_integrity_cbor() -> impl Future01<Item = (), Error = JsValue>
{
	let _ = console_log::init_with_level( Level::Trace );

	console_log!( "starting test: data_integrity_cbor" );

	let dataset: Vec<Data> = vec!
	[
		Data{ hello: "Hello CBOR - basic".to_string(), data: vec![ 0, 33245, 3, 36 ], num: 3948594 },

		// Test with something big
		//
		Data{ hello: "Hello CBOR - 1MB data".to_string(), data: vec![ 1; 1_024_000 ], num: 3948595 },
	];

	async move
	{
		for data in dataset
		{
			echo_cbor( data ).await;
		}

		let r: Result<(), wasm_bindgen::JsValue> = Ok(());

		r

	}.boxed_local().compat()
}


// Send data to an echo server and verify that what returns is exactly the same
//
async fn echo_cbor( data: Data )
{
	console_log!( "   Enter echo_cbor: {}", &data.hello );

	let (_ws , wsio) = connect().await;

	let codec: Codec<Data, Data>  = Codec::new().packed( true );
	let (tx, rx) = codec.framed( wsio ).split();

	let mut tx  = tx .sink_compat();
	let mut rx  = rx .compat();


	tx.send( data.clone() ).await.expect_throw( "Failed to write to websocket" );

	let msg = rx.next().await.unwrap_throw().unwrap_throw();

	assert_eq!( data, msg );
}



