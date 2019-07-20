#![ feature( async_await ) ]

use wasm_bindgen::prelude::*;


use
{
	async_runtime  :: { *          } ,
	web_sys        :: { *          } ,
	ws_stream_wasm :: { *          } ,
	futures_codec  :: { *          } ,
	futures        :: { prelude::*, stream::SplitStream } ,
	log :: { *                  } ,
};


const URL: &str = "ws://127.0.0.1:3412";


// Called when the wasm module is instantiated
//
#[ wasm_bindgen( start ) ]
//
pub fn main() -> Result<(), JsValue>
{
	let _ = console_log::init_with_level( Level::Trace );

	// Since there is no threads in wasm for the moment, this is optional if you include async_runtime
	// with `default-dependencies = false`, the local pool will be the default. However this might
	// change in the future.
	//
	rt::init( RtConfig::Local ).expect( "Set default executor" );

	let program = async move
	{
		let window   = web_sys::window  ().expect( "no global `window` exists"        );
		let document = window  .document().expect( "should have a document on window" );
		let _body    = document.body    ().expect( "document should have a body"      );
		let chat     = document.get_element_by_id( "chat" ).expect( "find chat"       );

		let (mut ws, wsio)  = WsStream::connect( URL, None ).await.expect_throw( "Could not create websocket" );
		let framed          = Framed::new( wsio, LinesCodec {} );
		let (mut out, msgs) = framed.split();


		rt::spawn_local( handle_msgs( msgs ) ).expect( "spawn msgs" );
	};




	rt::spawn_local( program ).expect( "spawn program" );

	Ok(())
}


fn append_line( document: &Document, chat: &Element, line: &str )
{
	let p = document.create_element( "p" ).expect( "Failed to create div" );

	p.set_inner_html( line );

	chat.append_child( &p ).expect( "Coundn't append child" );
}


async fn handle_msgs( stream: SplitStream< Framed<WsIo, LinesCodec> > )
{
	let window   = web_sys::window  ().expect( "no global `window` exists"        );
	let document = window  .document().expect( "should have a document on window" );
	let chat     = document.get_element_by_id( "chat" ).expect( "find chat"       );

	stream.for_each( |msg| async
	{{
		append_line( &document, &chat, &msg.expect_throw( "get msg" ) );

	}}).await;
}
