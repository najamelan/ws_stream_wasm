//! This is an echo server that returns all incoming bytes, without framing. It is used for the tests in
//! ws_stream_wasm.
//
#![ feature( async_closure ) ]

// mod error;
// use error::*;

use
{
	chat_format        :: { ClientMsg, ServerMsg                                       } ,
	futures_cbor_codec :: { Codec                                                      } ,
	chrono             :: { Utc                                                        } ,
	log                :: { *                                                          } ,
	regex              :: { Regex                                                      } ,
	ws_stream          :: { *                                                          } ,
	std                :: { env, collections::HashMap, net::SocketAddr                 } ,
	std::sync          :: { Arc, atomic::{ AtomicUsize, Ordering }                     } ,
	locks::rwlock      :: { RwLock, read::FutureReadable, write::FutureWriteable       } ,
	futures_codec      :: { Framed                                                     } ,
	futures            :: { select, sink::SinkExt, future::{ FutureExt, TryFutureExt } } ,
	warp               :: { Filter                                                     } ,
	once_cell          :: { sync::OnceCell                                             } ,
	pin_utils          :: { pin_mut                                                    } ,

	futures ::
	{
		StreamExt                                     ,
		channel::mpsc::{ unbounded, UnboundedSender } ,
	},
};


type ConnMap = HashMap<SocketAddr, Connection>;


struct Connection
{
	nick     : Arc<RwLock<String>>        ,
	sid      : usize                      ,
	tx       : UnboundedSender<ServerMsg> ,
}


static WELCOME : &str                         = "Welcome to the ws_stream Chat Server!" ;
static CONNS   :  OnceCell<RwLock< ConnMap >> = OnceCell::new();
static CLIENT  :  AtomicUsize                 = AtomicUsize::new(0);


fn main()
{
	flexi_logger::Logger::with_str( "server=trace, ws_stream=error, tokio=warn" ).start().unwrap();


	// This should be unfallible, so we ignore the result
	//
	let _ = CONNS.set( RwLock::new( HashMap::new() ) );


	// GET /chat -> websocket upgrade
	//
	let chat = warp::path( "chat" )

		// The `ws2()` filter will prepare Websocket handshake...
		//
		.and( warp::ws2() )

		.and( warp::addr::remote() )

		.map( |ws: warp::ws::Ws2, peer_addr: Option<SocketAddr>|
		{
			// This will call our function if the handshake succeeds.
			//
			ws.on_upgrade( move |socket|

				handle_conn( WarpWebSocket::new(socket), peer_addr.expect( "need peer_addr" )).boxed().compat()
			)
		})
	;

	// GET / -> index html
	//
	let index   = warp::path::end()        .and( warp::fs::file( "../chat_client/index.html" ) );
	let style   = warp::path( "style.css" ).and( warp::fs::file( "../chat_client/style.css"  ) );
	let statics = warp::path( "pkg" )      .and( warp::fs::dir ( "../chat_client/pkg"        ) );

	let routes  = index.or( style ).or( chat ).or( statics );


	let addr: SocketAddr = env::args().nth(1).unwrap_or( "127.0.0.1:3412".to_string() ).parse().expect( "valid addr" );
	println!( "server task listening at: {}", &addr );

	warp::serve( routes ).run( addr );
}



// Runs once for each incoming connection, ends when the stream closes or sending causes an
// error.
//
async fn handle_conn( socket: WarpWebSocket, peer_addr: SocketAddr ) -> Result<(), ()>
{
	let ws_stream = WsMeta::new( socket );

	info!( "Incoming connection from: {:?}", peer_addr );

	let (mut tx, rx)    = unbounded();
	let framed          = Framed::new( ws_stream, Codec::new() );
	let (out, mut msgs) = framed.split();
	let nick            = Arc::new( RwLock::new( peer_addr.to_string() ) );


	// A unique sender id for this client
	//
	let     sid: usize  = CLIENT.fetch_add( 1, Ordering::Relaxed );
	let mut joined      = false;

	let nick2 = nick.clone();

	// Listen to the channel for this connection and sends out each message that
	// arrives on the channel.
	//
	let outgoing = async move
	{
		// ignore result, just log
		//
		let _ =

			 rx
			.map     ( |res| Ok( res )       )
			.forward ( out                   )
			.await
			.map_err ( |e| error!( "{}", e ) )
		;
	};



	let incoming = async move
	{
		// Incoming messages. Ends when stream returns None or an error.
		//
		while let Some( msg ) = msgs.next().await
		{
			debug!( "received client message" );

			// TODO: handle io errors
			//
			let msg = match msg
			{
				Ok( msg ) => msg,
				_         => continue,
			};

			let time = Utc::now().timestamp();


			match msg
			{
				ClientMsg::Join( new_nick ) =>
				{
					debug!( "received Join" );

					// Clients should only join once. Since nothing bad can happen here,
					// we just ignore this message
					//
					if joined { continue; }

					let res = { validate_nick( sid, &nick.future_read().await.clone(), &new_nick ) };

					match res
					{
						Ok(_) =>
						{
							joined = true;
							*nick.future_write().await = new_nick.clone();

							// Let all clients know there is a new kid on the block
							//
							broadcast( &ServerMsg::UserJoined { time: Utc::now().timestamp(), nick: new_nick, sid } );

							// Welcome message
							//
							trace!( "sending welcome message" );


							let all_users =
							{
								let mut conns = CONNS.get().unwrap().future_write().await;

								conns.insert
								(
									peer_addr,
									Connection { tx: tx.clone(), nick: nick.clone(), sid },
								);

								conns.values().map( |c| (c.sid, c.nick.read().clone() )).collect()

							};

							let welcome = ServerMsg::Welcome
							{
								time : Utc::now().timestamp() ,
								txt  : WELCOME.to_string()    ,
								users: all_users              ,
							};

							send( sid, ServerMsg::JoinSuccess );
							send( sid, welcome );
						}

						Err( m ) =>
						{
							tx.send( m ).await.expect( "send on channel" );
						}
					}

				}


				ClientMsg::SetNick( new_nick ) =>
				{
					debug!( "received SetNick" );


					let res = validate_nick( sid, &nick.read(), &new_nick );

					debug!( "validated new Nick" );

					match res
					{
						Ok ( m ) =>
						{
							broadcast( &m );
							*nick.write() = new_nick;
						}

						Err( m ) => send( sid, m ),
					}
				}


				ClientMsg::ChatMsg( txt ) =>
				{
					debug!( "received ChatMsg" );

					broadcast( &ServerMsg::ChatMsg { time, nick: nick.future_read().await.clone(), sid, txt } );
				}
			}
		};

		debug!( "Broke from msgs loop" );
	};

	pin_mut!( incoming );
	pin_mut!( outgoing );

	select!
	{
		_ = incoming.fuse() => {}
		_ = outgoing.fuse() => {}
	};


	// remove the client and let other clients know this client disconnected
	//
	let user = { CONNS.get().unwrap().future_write().await.remove( &peer_addr ) };

	if user.is_some()
	{
		// let other clients know this client disconnected
		//
		broadcast( &ServerMsg::UserLeft { time: Utc::now().timestamp(), nick: nick2.future_read().await.clone(), sid } );


		debug!( "Client disconnected: {}", peer_addr );
	}


	info!( "End of connection from: {:?}", peer_addr );

	Ok(())

}



// Send a server message to all connected clients
//
fn broadcast( msg: &ServerMsg )
{
	debug!( "start broadcast" );

	for client in CONNS.get().unwrap().read().values().map( |c| &c.tx )
	{
		client.unbounded_send( msg.clone() ).expect( "send on unbounded" );
	};

	debug!( "finish broadcast" );
}



// Send a server message to all connected clients
//
fn send( sid: usize, msg: ServerMsg )
{
	debug!( "start send" );

	for client in CONNS.get().unwrap().read().values().filter( |c| c.sid == sid ).map( |c| &c.tx )
	{
		client.unbounded_send( msg.clone() ).expect( "send on unbounded" );
	};

	debug!( "finish send" );
}



// Send a server message to all connected clients
//
fn validate_nick( sid: usize, old: &str, new: &str ) -> Result<ServerMsg, ServerMsg>
{
	// Check whether it's unchanged
	//
	if old == new
	{
		return Err( ServerMsg::NickUnchanged{ time: Utc::now().timestamp(), sid, nick: old.to_string() } );
	}


	// Check whether it's not already in use
	//
	if CONNS.get().unwrap().read().values().any( |c| *c.nick.read() == new )
	{
		return Err( ServerMsg::NickInUse{ time: Utc::now().timestamp(), sid, nick: new.to_string() } )
	}


	// Check whether it's valid
	//
	let nickre   = Regex::new( r"^\w{1,15}$" ).unwrap();

	if !nickre.is_match( new )
	{
		error!( "Wrong nick: '{}'", new );
		return Err( ServerMsg::NickInvalid{ time: Utc::now().timestamp(), sid, nick: new.to_string() } )
	}


	// It's valid
	//
	Ok( ServerMsg::NickChanged{ time: Utc::now().timestamp(), old: old.to_string(), new: new.to_string(), sid } )
}



// fn random_id() -> String
// {
// 	thread_rng()
// 		.sample_iter( &Alphanumeric )
// 		.take(8)
// 		.collect()
// }

