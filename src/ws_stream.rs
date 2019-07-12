use
{
	crate :: { import::*, ChunkStream, JsWebSocket, WsReadyState },
};


type AsyncTokioResult = Result< Async<()>   , tokio::io::Error >;


/// A tokio AsyncRead/AsyncWrite representing a WebSocket connection. It only supports binary mode. Contrary to the rest of this library,
/// this will work on types from [futures 0.1](https://docs.rs/futures/0.1.25/futures/) instead of [0.3](https://rust-lang-nursery.github.io/futures-api-docs/0.3.0-alpha.13/futures/index.html). This is because tokio currently is on futures 0.1, so the stream returned from
/// a codec will be 0.1.
///
/// Currently !Sync and !Send.
///
/// ## Example
///
/// This example if from the integration tests. Uses [tokio-serde-cbor](https://docs.rs/tokio-serde-cbor/0.3.1/tokio_serde_cbor/) to send arbitrary data that implements [serde::Serialize](https://docs.rs/serde/1.0.89/serde/trait.Serialize.html) over a websocket.
///
/// ```
/// #[ derive( Debug, Clone, Serialize, Deserialize, PartialEq, Eq ) ]
/// //
/// struct Data
/// {
///    hello: String   ,
///    data : Vec<u32> ,
///    num  : u64      ,
/// }
///
/// let dataset: Vec<Data> = vec!
/// [
///    Data{ hello: "Hello CBOR - basic"   .to_string(), data: vec![ 0, 33245, 3, 36 ], num: 3948594 },
///    Data{ hello: "Hello CBOR - 4MB data".to_string(), data: vec![ 1; 1_024_000    ], num: 3948594 },
/// ];
///
/// let fut = async move
/// {
///    for data in dataset
///    {
///       echo_cbor( data ).await;
///    }
///
///    Ok(())
///
/// }.boxed().compat();
///
/// spawn_local( fut );
///
///
///
/// // Send data to an echo server and verify that what returns is exactly the same
/// //
/// async fn echo_cbor( data: Data )
/// {
///    console_log!( "   Enter echo_cbor: {}", &data.hello );
///
///    let ws = connect().await;
///
///    let codec: Codec<Data, Data> = Codec::new().packed( true );
///    let (tx, mut rx) = codec.framed( ws ).split();
///
///    tx.send( data.clone() ).await.expect_throw( "Failed to write to websocket" );
///
///    let msg    = rx.next().await;
///    let result = &mut msg.unwrap_throw().unwrap_throw();
///
///    assert_eq!( &data, result );
/// }
/// ```
///
#[ allow( dead_code ) ] // for the on_mesg...
//
#[ derive( Debug ) ]
//
pub struct WsStream
{
	// RefCell because we want to be able to read on a not mutable reference of WsStream.
	// And because the closure also accesses this property.
	//
	incoming : Rc<RefCell< ChunkStream >>        ,
	ws       : JsWebSocket                       ,

	// The task to be notified if we receive new data
	//
	task     : Rc<RefCell< Option<task::Task> >> ,
}


impl WsStream
{
	/// Create a new WsStream. The future resolves when the connection is established.
	///
	pub async fn connect< T: AsRef<str> >( url: T ) -> Result< WsStream, JsValue >
	{
		let task: Rc<RefCell<Option<task::Task>>> = Rc::new( RefCell::new( None ) ) ;
		let t2                                    = task.clone();

		let incoming = Rc::new( RefCell::new( ChunkStream::new() ) );
		let i2       = incoming.clone();



		let cb = Box::new( move |msg_evt: MessageEvent|
		{
			trace!( "WsStream: message received!" );


			let data = msg_evt.data();

			if data.is_instance_of::< ArrayBuffer >()
			{
				let raw = data.dyn_into::< ArrayBuffer >().unwrap_throw();

				i2.borrow_mut().push( Uint8Array::new( raw.as_ref() ) );
			}

			else if data.is_string()
			{
				i2.borrow_mut().push( Uint8Array::new(&data) )
			}

			else
			{
				error!( "WsStream: Invalid data format" );
			};


			if let Some( ref t ) = *t2.borrow()
			{
				trace!( "WsStream: waking up task" );
				t.notify()
			}
		});

		let ws = JsWebSocket::with_on_message( url.as_ref(), cb )? ;


		ws.connect().await;

		Ok( Self { ws, task, incoming } )
	}



	/// Verify the [WsReadyState] of the connection.
	///
	pub fn state( &self ) -> WsReadyState { self.ws.ready_state() }



	/// Get the url of the server we are connected to.
	///
	pub fn url  ( &self ) -> String       { self.ws.url()         }



	//---------- impl io::Read
	//
	fn io_read( &self, buf: &mut [u8] ) -> Result< usize, io::Error >
	{
		trace!( "WsStream: read called" );

		let mut inc = self.incoming.borrow_mut();

		if inc.is_empty()
		{
			trace!( "WsStream: read would block" );

			*self.task.borrow_mut() = Some( task::current() );

			Err( io::Error::from( WouldBlock ) )
		}

		else
		{
			Ok( inc.read( 0, buf ) as usize )
		}
	}



	// -------io:Write impl
	//
	fn io_write( &self, buf: &[u8] ) -> io::Result< usize >
	{
		// FIXME: avoid extra copy? Probably use one of the other methods on WebSocket, like
		// send_with_array_buffer_view. We have to figure out how to create those from a &[u8]
		//
		let result = self.ws.wrapped().send_with_u8_array( &mut Vec::from( buf ) );

		match result
		{
			Ok (_ ) => Ok ( buf.len()                                      ) ,
			Err(_e) => Err( io::Error::from( io::ErrorKind::NotConnected ) ) ,
		}
	}


	fn io_flush( &self ) -> io::Result<()>
	{
		Ok(())
	}


	// -------AsyncWrite impl
	//
	fn async_shutdown( &self ) -> AsyncTokioResult
	{
		// This can not throw normally, because the only errors the api
		// can return is if we use a code or a reason string, which we don't.
		//
		self.ws.wrapped().close().unwrap_throw();

		Ok(().into())
	}
}


impl Drop for WsStream
{
	fn drop( &mut self )
	{
		// This can not throw normally, because the only errors the api
		// can return is if we use a code or a reason string, which we don't.
		//
		self.ws.wrapped().close().unwrap_throw();
	}
}



// FIXME: Can we not do this by just implementing Deref? It seems dumb to have all this boilerplate.
//        When I checked tokio code for (Tcp, Uds, ...) they all seem to have 2 copies for each impl,
//        not at all keeping it DRY. At least here we refactor those into impl WsStream... I'm a bit
//        confused by this impls for references.

impl     io::Read for     WsStream { fn read( &mut self, buf: &mut [u8] ) -> Result< usize, io::Error > { self.io_read( buf ) } }
impl<'a> io::Read for &'a WsStream { fn read( &mut self, buf: &mut [u8] ) -> Result< usize, io::Error > { self.io_read( buf ) } }


impl     io::Write for     WsStream
{
	fn write( &mut self, buf: &[u8] ) -> io::Result< usize > { self.io_write( buf ) }
	fn flush( &mut self             ) -> io::Result< ()    > { self.io_flush(     ) }
}

impl<'a> io::Write for &'a WsStream
{
	fn write( &mut self, buf: &[u8] ) -> io::Result< usize > { self.io_write( buf ) }
	fn flush( &mut self             ) -> io::Result< ()    > { self.io_flush(     ) }
}


impl     AsyncRead01 for           WsStream   {}
impl<'a> AsyncRead01 for &'a       WsStream   {}


impl     AsyncWrite01 for     WsStream { fn shutdown( &mut self ) -> AsyncTokioResult {  self.async_shutdown() } }
impl<'a> AsyncWrite01 for &'a WsStream { fn shutdown( &mut self ) -> AsyncTokioResult {  self.async_shutdown() } }
