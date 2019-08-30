#![ allow( unused_imports ) ]

pub(crate) mod e_handler ;
pub(crate) mod color     ;
pub(crate) mod user_list ;



mod import
{
	pub(crate) use
	{
		chat_format          :: { ServerMsg, ClientMsg                                                  } ,
		async_runtime        :: { *                                                                     } ,
		web_sys              :: { *, console::log_1 as dbg                                              } ,
		ws_stream_wasm       :: { *                                                                     } ,
		futures_codec        :: { *                                                                     } ,
		futures_cbor_codec   :: { Codec, Error as CodecError                                            } ,
		futures              :: { prelude::*, stream::SplitStream, select, ready                        } ,
		futures              :: { channel::{ mpsc::{ unbounded, UnboundedReceiver, UnboundedSender } }  } ,
		log                  :: { *                                                                     } ,
		web_sys              :: { *                                                                     } ,
		wasm_bindgen         :: { prelude::*, JsCast                                                    } ,
		gloo_events          :: { *                                                                     } ,
		std                  :: { rc::Rc, convert::TryInto, cell::RefCell, io                           } ,
		std                  :: { task::*, pin::Pin, collections::HashMap, panic                        } ,
		regex                :: { Regex                                                                 } ,
		js_sys               :: { Date, Math                                                            } ,
		pin_utils            :: { pin_mut                                                               } ,
		wasm_bindgen_futures :: { futures_0_3::spawn_local                                              } ,
	};
}

use crate::{ import::*, e_handler::*, color::*, user_list::* };


const HELP: &str = "Available commands:
/nick NEWNAME # change nick (must be between 1 and 15 word characters)
/help # Print available commands";


// Called when the wasm module is instantiated
//
#[ wasm_bindgen( start ) ]
//
pub fn main() -> Result<(), JsValue>
{
	panic::set_hook(Box::new(console_error_panic_hook::hook));

	// Let's only log output when in debug mode
	//
	#[ cfg( debug_assertions ) ]
	//
	wasm_logger::init( wasm_logger::Config::new(Level::Debug).message_on_new_line() );

	let program = async
	{
		let cform = get_id( "connect_form" );
		let tarea = get_id( "chat_input"   );
		let chat  = get_id( "chat_form"    );

		let cnick: HtmlInputElement = get_id( "connect_nick" ).unchecked_into();
		cnick.set_value( random_name() );

		let enter_evts   = EHandler::new( &tarea, "keypress", false );
		let csubmit_evts = EHandler::new( &cform, "submit"  , false );
		let creset_evts  = EHandler::new( &cform, "reset"   , false );
		let reset_evts   = EHandler::new( &chat , "reset"   , false );

		let (tx, rx) = unbounded();


		spawn_local( on_disconnect( reset_evts, rx ) );
		spawn_local( on_key       ( enter_evts     ) );
		spawn_local( on_cresets   ( creset_evts    ) );

		on_connect( csubmit_evts, tx ).await;

		info!( "main function ends" );
	};

	spawn_local( program );


	Ok(())
}


fn append_line( chat: &Element, time: f64, nick: &str, line: &str, color: &Color, color_all: bool )
{
	let p: HtmlElement = document().create_element( "p"    ).expect_throw( "create p"    ).unchecked_into();
	let n: HtmlElement = document().create_element( "span" ).expect_throw( "create span" ).unchecked_into();
	let m: HtmlElement = document().create_element( "span" ).expect_throw( "create span" ).unchecked_into();
	let t: HtmlElement = document().create_element( "span" ).expect_throw( "create span" ).unchecked_into();

	n.style().set_property( "color", &color.to_css() ).expect_throw( "set color" );

	if color_all
	{
		m.style().set_property( "color", &color.to_css() ).expect_throw( "set color" );
	}

	// Js needs milliseconds, where the server sends seconds
	//
	let time = Date::new( &( time * 1000 as f64 ).into() );

	n.set_inner_text( &format!( "{}: ", nick )                                                                     );
	m.set_inner_text( line                                                                                         );
	t.set_inner_text( &format!( "{:02}:{:02}:{:02} - ", time.get_hours(), time.get_minutes(), time.get_seconds() ) );

	n.set_class_name( "nick"         );
	m.set_class_name( "message_text" );
	t.set_class_name( "time"         );

	p.append_child( &t ).expect( "Coundn't append child" );
	p.append_child( &n ).expect( "Coundn't append child" );
	p.append_child( &m ).expect( "Coundn't append child" );

	// order is important here, we need to measure the scroll before adding the item
	//
	let max_scroll = chat.scroll_height() - chat.client_height();
	chat.append_child( &p ).expect( "Coundn't append child" );

	// Check whether we are scolled to the bottom. If so, we autoscroll new messages
	// into vies. If the user has scrolled up, we don't.
	//
	// We keep a margin of up to 2 pixels, because sometimes the two numbers don't align exactly.
	//
	if ( chat.scroll_top() - max_scroll ).abs() < 3
	{
		p.scroll_into_view();
	}
}


async fn on_msg( mut stream: impl Stream<Item=Result<ServerMsg, CodecError>> + Unpin )
{
	let chat       = get_id( "chat" );
	let mut u_list = UserList::new();

	let mut colors: HashMap<usize, Color> = HashMap::new();

	// for the server messages
	//
	colors.insert( 0, Color::random().light() );


	while let Some( msg ) = stream.next().await
	{
		// TODO: handle io errors
		//
		let msg = match msg
		{
			Ok( msg ) => msg,
			_         => continue,
		};



		debug!( "received message" );


		match msg
		{
			ServerMsg::ChatMsg{ time, nick, sid, txt } =>
			{
				let color = colors.entry( sid ).or_insert( Color::random().light() );

				append_line( &chat, time as f64, &nick, &txt, color, false );
			}


			ServerMsg::ServerMsg{ time, txt } =>
			{
				append_line( &chat, time as f64, "Server", &txt, colors.get( &0 ).unwrap(), true );
			}


			ServerMsg::Welcome{ time, users, txt } =>
			{
				users.into_iter().for_each( |(s,n)| u_list.insert(s,n) );

				let udiv = get_id( "users" );
				u_list.render( udiv.unchecked_ref() );

				append_line( &chat, time as f64, "Server", &txt, colors.get( &0 ).unwrap(), true );


				// Client Welcome message
				//
				append_line( &chat, time as f64, "ws_stream_wasm Client", HELP, colors.get( &0 ).unwrap(), true );
			}


			ServerMsg::NickChanged{ time, old, new, sid } =>
			{
				append_line( &chat, time as f64, "Server", &format!( "{} has changed names => {}.", &old, &new ), colors.get( &0 ).unwrap(), true );
				u_list.insert( sid, new );
			}


			ServerMsg::NickUnchanged{ time, .. } =>
			{
				append_line( &chat, time as f64, "Server", "Error: You specified your old nick instead of your new one, it's unchanged.", colors.get( &0 ).unwrap(), true );
			}


			ServerMsg::NickInUse{ time, nick, .. } =>
			{
				append_line( &chat, time as f64, "Server", &format!( "Error: The nick is already in use: '{}'.", &nick ), colors.get( &0 ).unwrap(), true );
			}


			ServerMsg::NickInvalid{ time, nick, .. } =>
			{
				append_line( &chat, time as f64, "Server", &format!( "Error: The nick you specify must be between 1 and 15 word characters, was invalid: '{}'.", &nick ), colors.get( &0 ).unwrap(), true );
			}


			ServerMsg::UserJoined{ time, nick, sid } =>
			{
				append_line( &chat, time as f64, "Server", &format!( "We welcome a new user, {}!", &nick ), colors.get( &0 ).unwrap(), true );
				u_list.insert( sid, nick );
			}


			ServerMsg::UserLeft{ time, nick, sid } =>
			{
				u_list.remove( sid );
				append_line( &chat, time as f64, "Server", &format!( "Sadly, {} has left us.", &nick ), colors.get( &0 ).unwrap(), true );
			}

			_ => {}
		}
	}

	// The stream has closed, so we are disconnected
	//
	show_connect_form();

	debug!( "leaving on_msg" );
}


async fn on_submit
(
	mut submits: impl Stream< Item=Event                  > + Unpin ,
	mut out    : impl Sink  < ClientMsg, Error=CodecError > + Unpin ,
)
{
	let chat     = get_id( "chat" );

	let nickre   = Regex::new( r"^/nick (\w{1,15})" ).unwrap();

	// Note that we always add a newline below, so we have to match it.
	//
	let helpre   = Regex::new(r"^/help\n$").unwrap();

	let textarea = get_id( "chat_input" );
	let textarea: &HtmlTextAreaElement = textarea.unchecked_ref();


	while let Some( evt ) = submits.next().await
	{
		debug!( "on_submit" );

		evt.prevent_default();

		let text = textarea.value().trim().to_string() + "\n";
		textarea.set_value( "" );
		let _ = textarea.focus();

		if text == "\n" { continue; }

		let msg;


		// if this is a /nick somename message
		//
		if let Some( cap ) = nickre.captures( &text )
		{
			debug!( "handle set nick: {:#?}", &text );

			msg = ClientMsg::SetNick( cap[1].to_string() );
		}


		// if this is a /help message
		//
		else if helpre.is_match( &text )
		{
			debug!( "handle /help: {:#?}", &text );

			append_line( &chat, Date::now(), "ws_stream_wasm Client", HELP, &Color::random().light(), true );

			return;
		}


		else
		{
			debug!( "handle send: {:#?}", &text );

			msg = ClientMsg::ChatMsg( text );
		}


		match out.send( msg ).await
		{
			Ok(()) => {}

			Err(e) =>
			{
				match e
				{
					// We lost the connection to the server
					//
					CodecError::Io(err) => match err.kind()
					{
						io::ErrorKind::NotConnected =>
						{
							error!( "The connection to the server was lost" );

							// Show login screen...
							//
							show_connect_form();
						}

						_ => error!( "{}", &err ),

					},

					_ => error!( "{}", &e ),
				}
			}
		};
	};
}




// When the user presses the Enter key in the textarea we submit, rather than adding a new line
// for newline, the user can use shift+Enter.
//
// We use the click effect on the Send button, because form.submit() wont let our on_submit handler run.
//
async fn on_key
(
	mut keys: impl Stream< Item=Event > + Unpin ,
)
{
	let send: HtmlElement = get_id( "chat_submit" ).unchecked_into();


	while let Some( evt ) = keys.next().await
	{
		let evt: KeyboardEvent = evt.unchecked_into();

		if  evt.code() == "Enter"  &&  !evt.shift_key()
		{
			send.click();
			evt.prevent_default();
		}
	};
}



async fn on_connect( mut evts: impl Stream< Item=Event > + Unpin, mut disconnect: UnboundedSender<WsStream> )
{
	while let Some(evt) = evts.next().await
	{
		info!( "Connect button clicked" );

		evt.prevent_default();

		// validate form
		//
		let (nick, url) = match validate_connect_form()
		{
			Ok(ok) => ok,

			Err( _ ) =>
			{
				// report error to the user
				// continue loop
				//
				unreachable!()
			}
		};


		let (ws, wsio) = match WsStream::connect( url, None ).await
		{
			Ok(conn) => conn,

			Err(e)   =>
			{
				// report error to the user
				//
				error!( "{}", e );
				continue;
			}
		};

		let framed              = Framed::new( wsio, Codec::new() );
		let (mut out, mut msgs) = framed.split();

		let form    = get_id( "chat_form" );
		let on_send = EHandler::new( &form, "submit", false );


		// hide the connect form
		//
		let cform: HtmlElement = get_id( "connect_form" ).unchecked_into();


		// Ask the server to join
		//
		match out.send( ClientMsg::Join( nick ) ).await
		{
			Ok(()) => {}
			Err(e) => { error!( "{}", e ); }
		};


		// Error handling
		//
		let cerror: HtmlElement = get_id( "connect_error" ).unchecked_into();


		if let Some(response) = msgs.next().await
		{
			match response
			{
				Ok( ServerMsg::JoinSuccess ) =>
				{
					info!( "got joinsuccess" );

					cform.style().set_property( "display", "none" ).expect_throw( "set cform display none" );

					disconnect.send( ws ).await.expect_throw( "send ws to disconnect" );

					let msg = on_msg   ( msgs          ).fuse();
					let sub = on_submit( on_send , out ).fuse();

					pin_mut!( msg );
					pin_mut!( sub );

					// on_msg will end when the stream get's closed by disconnect. This way we will
					// stop on_submit as well.
					//
					select!
					{
						_ = msg => {}
						_ = sub => {}
					}
	  			}

				// Show an error message on the connect form and let the user try again
				//
				Ok( ServerMsg::NickInUse{ .. } ) =>
				{
					cerror.set_inner_text( "The nick name is already in use. Please choose another." );
					cerror.style().set_property( "display", "block" ).expect_throw( "set display block on cerror" );

					continue;
				}

				Ok( ServerMsg::NickInvalid{ .. } ) =>
				{
					cerror.set_inner_text( "The nick name is invalid. It must be between 1 and 15 word characters." );
					cerror.style().set_property( "display", "block" ).expect_throw( "set display block on cerror" );

					continue;
				}

				// cbor decoding error
				//
				Err(e) =>
				{
					error!( "{}", e );
				}

				_ => {  }
			}
		}
	}

	error!( "on_connect ends" );
}



fn validate_connect_form() -> Result< (String, String), () >
{
	let nick_field: HtmlInputElement = get_id( "connect_nick" ).unchecked_into();
	let url_field : HtmlInputElement = get_id( "connect_url"  ).unchecked_into();

	let nick = nick_field.value();
	let url  = url_field .value();

	Ok((nick, url))
}



async fn on_cresets( mut evts: impl Stream< Item=Event > + Unpin )
{
	while let Some( evt ) = evts.next().await
	{
		evt.prevent_default();

		let cnick: HtmlInputElement = get_id( "connect_nick" ).unchecked_into();
		let curl : HtmlInputElement = get_id( "connect_url"  ).unchecked_into();

		cnick.set_value( random_name()              );
		curl .set_value( "ws://127.0.0.1:3412/chat" );
	}
}



async fn on_disconnect( mut evts: impl Stream< Item=Event > + Unpin, mut wss: UnboundedReceiver<WsStream> )
{
	let ws1: Rc<RefCell<Option<WsStream>>> = Rc::new(RefCell::new( None ));
	let ws2 = ws1.clone();


	let wss_in = async move
	{
		while let Some( ws ) = wss.next().await
		{
			*ws2.borrow_mut() = Some(ws);
		}
	};

	spawn_local( wss_in );


	while evts.next().await.is_some()
	{
		debug!( "on_disconnect" );

		if let Some( ws_stream ) = ws1.borrow_mut().take()
		{
			ws_stream.close().await.expect_throw( "close ws" );
			debug!( "connection closed by disconnect" );
			show_connect_form();
		}
	}
}



fn show_connect_form()
{
	// show the connect form
	//
	let cform: HtmlElement = get_id( "connect_form" ).unchecked_into();

	cform.style().set_property( "display", "flex" ).expect_throw( "set cform display none" );

	get_id( "users" ).set_inner_html( "" );
	get_id( "chat"  ).set_inner_html( "" );
}



pub fn document() -> Document
{
	let window = web_sys::window().expect_throw( "no global `window` exists");

	window.document().expect_throw( "should have a document on window" )
}



// Return a random name
//
pub fn random_name() -> &'static str
{
	// I wanted to use the crate scottish_names to generate a random username, but
	// it uses the rand crate which doesn't support wasm for now, so we're just using
	// a small sample.
	//
	let list = vec!
	[
		  "Aleeza"
		, "Aoun"
		, "Arya"
		, "Azaan"
		, "Ebony"
		, "Emke"
		, "Elena"
		, "Hafsa"
		, "Hailie"
		, "Inaaya"
		, "Iqra"
		, "Kobi"
		, "Noor"
		, "Nora"
		, "Nuala"
		, "Orin"
		, "Pippa"
		, "Rhuaridh"
		, "Salah"
		, "Susheela"
		, "Teya"
	];

	// pick one
	//
	list[ Math::floor( Math::random() * list.len() as f64 ) as usize ]
}


fn get_id( id: &str ) -> Element
{
	document().get_element_by_id( id ).expect_throw( &format!( "find {}", id ) )
}





