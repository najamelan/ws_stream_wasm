use crate::{ import::* };


/// Events related to the WebSocket. You can filter like:
///
/// ```
/// use
///{
///   ws_stream_wasm       :: *                        ,
///   pharos               :: *                        ,
///   wasm_bindgen         :: UnwrapThrowExt           ,
///   wasm_bindgen_futures :: futures_0_3::spawn_local ,
///   futures              :: stream::StreamExt        ,
///};
///
///let program = async
///{
///   let (mut ws, _wsio) = WsMeta::connect( "127.0.0.1:3012", None ).await
///
///      .expect_throw( "assume the connection succeeds" );
///
///   // The Filter type comes from the pharos crate.
///   //
///   let mut evts = ws.observe( Filter::Pointer( WsEvent::is_closed ).into() );
///
///   ws.close().await;
///
///   // Note we will only get the closed event here, the WsEvent::Closing has been filtered out.
///   //
///   assert!( evts.next().await.unwrap_throw().is_closed () );
///};
///
///spawn_local( program );
///```
//
#[ derive( Clone, Debug, PartialEq, Eq ) ]
//
pub enum WsEvent
{
	/// The connection is now Open
	//
	Open,

	/// An error happened on the connection. For more information about when this event
	/// occurs, see the [HTML Living Standard](https://html.spec.whatwg.org/multipage/web-sockets.html).
	//
	Error,

	/// The connection has started closing, but is not closed yet.
	//
	Closing,

	/// The connection was closed. This enclosed CloseEvent has some extra information.
	//
	Closed( CloseEvent ),
}


impl WsEvent
{
	/// Predicate indicating whether this is a [WsEvent::Open] event. Can be used as a filter for the
	/// event stream obtained with [`WsMeta::observe`].
	//
	pub fn is_open( &self ) -> bool
	{
		match self
		{
			Self::Open => true,
			_          => false,
		}
	}

	/// Predicate indicating whether this is a [WsEvent::Error] event. Can be used as a filter for the
	/// event stream obtained with [`WsMeta::observe`].
	//
	pub fn is_err( &self ) -> bool
	{
		match self
		{
			Self::Error => true,
			_           => false,
		}
	}

	/// Predicate indicating whether this is a [WsEvent::Closing] event. Can be used as a filter for the
	/// event stream obtained with [`WsMeta::observe`].
	//
	pub fn is_closing( &self ) -> bool
	{
		match self
		{
			Self::Closing => true,
			_             => false,
		}
	}

	/// Predicate indicating whether this is a [WsEvent::Closed] event. Can be used as a filter for the
	/// event stream obtained with [`WsMeta::observe`].
	//
	pub fn is_closed( &self ) -> bool
	{
		match self
		{
			Self::Closed(_) => true,
			_               => false,
		}
	}
}



/// An event holding information about how the connection was closed.
///
// We use this wrapper because the web_sys version isn't Send and pharos requires events
// to be Send.
//
#[ derive( Clone, Debug, PartialEq, Eq ) ]
//
pub struct CloseEvent
{
	/// The close code
	/// See: [MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/close)
	//
	pub code     : u16    ,

	/// The reason why the connection was closed
	/// See: [MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/close)
	//
	pub reason   : String ,

	/// Whether the connection was closed cleanly
	/// See: [MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/close)
	//
	pub was_clean: bool   ,
}


impl From<JsCloseEvt> for CloseEvent
{
	fn from( js_evt: JsCloseEvt ) -> Self
	{
		Self
		{
			code     : js_evt.code     () ,
			reason   : js_evt.reason   () ,
			was_clean: js_evt.was_clean() ,
		}
	}
}




