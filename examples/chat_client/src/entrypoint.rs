#![ feature( async_await ) ]
#![ allow( unused_imports, unused_variables ) ]

use
{
	chat_format    :: { futures_serde_cbor::{ Codec, Error }, ChatMessage, Command } ,
	async_runtime  :: { *                                                          } ,
	web_sys        :: { *, console::log_1 as dbg                                   } ,
	ws_stream_wasm :: { *                                                          } ,
	futures_codec  :: { *                                                          } ,
	futures        :: { prelude::*, stream::SplitStream                            } ,
	futures        :: { channel::mpsc, ready                                       } ,
	log            :: { *                                                          } ,
	web_sys        :: {                                                            } ,
	wasm_bindgen   :: { prelude::*, JsCast                                         } ,
	gloo_events    :: { *                                                          } ,
	std            :: { task::*, pin::Pin, collections::HashMap, panic             } ,
	regex          :: { Regex                                                      } ,
	js_sys         :: { Math                                                       } ,
};


const URL: &str = "ws://127.0.0.1:3412";


// Called when the wasm module is instantiated
//
#[ wasm_bindgen( start ) ]
//
pub fn main() -> Result<(), JsValue>
{
	panic::set_hook(Box::new(console_error_panic_hook::hook));

	wasm_logger::init(
	    wasm_logger::Config::new(Level::Debug)
	        .message_on_new_line()
	);

	// Since there is no threads in wasm for the moment, this is optional if you include async_runtime
	// with `default-dependencies = false`, the local pool will be the default. However this might
	// change in the future.
	//
	rt::init( RtConfig::Local ).expect( "Set default executor" );

	let program = async
	{
		let window   = web_sys::window  ().expect( "no global `window` exists"        );
		let document = window  .document().expect( "should have a document on window" );
		let _body    = document.body    ().expect( "document should have a body"      );
		let chat     = document.get_element_by_id( "chat" ).expect( "find chat"       );

		let (ws, wsio) = match WsStream::connect( URL, None ).await
		{
			Ok(conn) => conn,
			Err(e)   =>
			{
				error!( "{}", e );
				return;
			}
		};

		let framed      = Framed::new( wsio, Codec::new() );
		let (out, msgs) = framed.split();

		let send    = document.get_element_by_id( "chat_submit" ).expect( "find chat_submit" );
		let form    = document.get_element_by_id( "chat_form" ).expect( "find chat_form" );
		let on_send = EHandler::new( &form, "reset", true );

		rt::spawn_local( handle_msgs( msgs         ) ).expect( "spawn msgs" );
		rt::spawn_local( handle_send( on_send, out ) ).expect( "spawn send" );
	};

	rt::spawn_local( program ).expect( "spawn program" );

	Ok(())
}


fn append_line( document: &Document, chat: &Element, line: &str, nick: &str, color: &Color, color_all: bool )
{
	let p: HtmlElement = document.create_element( "p"    ).expect( "Failed to create div" ).unchecked_into();
	let s: HtmlElement = document.create_element( "span" ).expect( "Failed to create div" ).unchecked_into();
	let t: HtmlElement = document.create_element( "span" ).expect( "Failed to create div" ).unchecked_into();

	debug!( "setting color to: {}", color.to_css() );

	s.style().set_property( "color", &color.to_css() ).expect_throw( "set color" );

	if color_all
	{
		t.style().set_property( "color", &color.to_css() ).expect_throw( "set color" );
	}


	s.set_inner_text( &( nick.to_string() + ": " ) );
	t.set_inner_text( line );
	s.set_class_name( "nick" );
	t.set_class_name( "message_text" );

	p.append_child( &s ).expect( "Coundn't append child" );
	p.append_child( &t ).expect( "Coundn't append child" );
	chat.append_child( &p ).expect( "Coundn't append child" );

	p.scroll_into_view();
}


async fn handle_msgs( mut stream: impl Stream<Item=Result<ChatMessage, Error>> + Unpin )
{
	let window   = web_sys::window  ().expect_throw( "no global `window` exists"        );
	let document = window  .document().expect_throw( "should have a document on window" );
	let chat     = document.get_element_by_id( "chat" ).expect_throw( "find chat"       );
	let re       = Regex::new( r"^#(\w{8})(.*)$" );

	let mut colors: HashMap<usize, Color> = HashMap::new();

	// for the server messages
	//
	colors.insert( 0, Color::random().light() );


	while let Some( msg ) = stream.next().await
	{
		let msg: ChatMessage = msg.expect_throw( "get msg" );

		debug!( "received message" );


		match msg.cmd
		{
			Command::Message       =>
			{
				let sid = msg.sid.unwrap();
				if ! colors.contains_key( &sid ) { colors.insert( sid, Color::random().light() ); }
				append_line( &document, &chat, &msg.txt.unwrap(), &msg.nick.unwrap(), colors.get( &sid ).unwrap(), false );
			}

			Command::ServerMessage =>
			{
				append_line( &document, &chat, &msg.txt.unwrap(), &msg.nick.unwrap(), colors.get( &0 ).unwrap(), true );
			}

			_ => {}
		}
	}
}


async fn handle_send
(
	mut on_clicks: impl Stream<Item=()> + Unpin,
	mut out      : impl Sink<ChatMessage, Error=Error> + Unpin
)
{
	let window   = web_sys::window  ().expect_throw( "no global `window` exists"              );
	let document = window  .document().expect_throw( "should have a document on window"       );
	let chat     = document.get_element_by_id( "chat" ).expect_throw( "find chat"       );
	let nickre   = Regex::new(r"^/nick (\w{1,15})").unwrap();
	let textarea = document.get_element_by_id( "chat_input" ).expect_throw( "find chat_input" );
	let textarea: &HtmlTextAreaElement = textarea.unchecked_ref();


	while on_clicks.next().await.is_some()
	{
		debug!( "click detected" );

		let text = textarea.value().trim().to_string() + "\n";
		// textarea.set_text_content( None );
		//

		if text == "\n" { continue; }

		let msg;

		if let Some( cap ) = nickre.captures( &text )
		{
			debug!( "handle set nick: {:#?}", &text );

			msg = ChatMessage
			{
				cmd : Command::SetNick           ,
				txt : Some( cap[1].to_string() ) ,
				nick: None                       ,
				sid : None                       ,
			};
		}


		else
		{
			debug!( "handle send: {:#?}", &text );

			msg = ChatMessage
			{
				cmd : Command::Message,
				txt : Some( text )    ,
				nick: None            ,
				sid : None            ,
			};
		}



		match out.send( msg ).await
		{
			Ok(()) => {}
			Err(e) => { error!( "{}", e ); }
		};
	};
}



pub struct EHandler
{
	receiver: mpsc::UnboundedReceiver<()>,

	// Automatically removed from the DOM on drop!
	//
	_listener: EventListener,
}


impl EHandler
{
	pub fn new( target: &EventTarget, event: &'static str, prevent_default: bool ) -> Self
	{
		debug!( "set onclick handler" );

		let (sender, receiver) = mpsc::unbounded();
		let options = match prevent_default
		{
			true  => EventListenerOptions::enable_prevent_default(),
			false => EventListenerOptions::default(),
		};

		// Attach an event listener
		//
		let _listener = EventListener::new_with_options( &target, event, options, move |_|
		{
			sender.unbounded_send(()).unwrap_throw();
		});

		Self
		{
			receiver,
			_listener,
		}
	}
}



impl Stream for EHandler
{
	type Item = ();

	fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>>
	{
		Pin::new( &mut self.receiver ).poll_next(cx)
	}
}



pub struct Color
{
	r: u8,
	g: u8,
	b: u8,
	a: u8,
}


impl Color
{
	pub fn new( r: u8, g: u8, b: u8, a: u8 ) -> Self
	{
		Self{ r, g, b, a }
	}

	pub fn random() -> Self
	{
		Self
		{
			r: ( Math::random() * 255_f64 ) as u8,
			g: ( Math::random() * 255_f64 ) as u8,
			b: ( Math::random() * 255_f64 ) as u8,
			a: ( Math::random() * 255_f64 ) as u8,
		}
	}


	// If this color is darker than half luminosity, it will be inverted
	//
	pub fn light( self ) -> Self
	{
		if self.is_dark() { self.invert() }
		else { self }
	}


	// If this color is lighter than half luminosity, it will be inverted
	//
	pub fn dark( self ) -> Self
	{
		if self.is_light() { self.invert() }
		else { self }
	}


	/// Invert color.
	//
	pub fn invert( mut self ) -> Self
	{
		self.r = 255 - self.r;
		self.g = 255 - self.g;
		self.b = 255 - self.b;
		self.a = 255 - self.a;

		self
	}


	// True if this color is lighter than half luminosity.
	//
	pub fn is_light( &self ) -> bool
	{
		self.r as u16 + self.g as u16 + self.b as u16 > 378 // 128 * 3
	}


	/// True if this color is darker than half luminosity.
	//
	pub fn is_dark( &self ) -> bool
	{
		!self.is_light()
	}


	// output a css string format: "#rrggbb"
	//
	pub fn to_css( &self ) -> String
	{
		format!( "#{:2x}{:2x}{:2x}", self.r, self.g, self.b )
	}


	// output a css string format: "rgba( rrr, ggg, bbb, aaa )"
	//
	pub fn to_cssa( &self ) -> String
	{
		format!( "rgba({},{},{},{})", self.r, self.g, self.b, self.a )
	}
}



