use crate::{ import::*, CloseEvent };

/// The error type for errors happening in `ws_stream_wasm`.
///
/// Use [`WsErr::kind()`] to know which kind of error happened.
//
#[ derive( Debug ) ]
//
pub struct WsErr
{
	inner: FailContext<WsErrKind>,
}



/// The different kind of errors that can happen when you use the `ws_stream_wasm` API.
//
#[ derive( Clone, PartialEq, Eq, Debug, Fail ) ]
//
pub enum WsErrKind
{
	/// Invalid input to [WsState::try_from( u16 )](crate::WsState)
	///
	#[ fail( display = "Invalid input to conversion to WsReadyState: {}", _0 ) ]
	//
	InvalidWsState( u16 ),

	/// When trying to send and [WsState](crate::WsState) is anything but [WsState::Open](crate::WsState::Open) this error is returned.
	//
	#[ fail( display = "The connection state is not \"Open\"" ) ]
	//
	ConnectionNotOpen,

	/// Browsers will forbid making websocket connections to certain ports. See this [Stack Overflow question](https://stackoverflow.com/questions/4313403/why-do-browsers-block-some-ports/4314070).
	///
	#[ fail( display = "The port to which the connection is being attempted is not allowed." ) ]
	//
	ForbiddenPort,

	/// An invalid url was given to [WsStream::connect](crate::WsStream::connect), please see:
	/// [HTML Living Standard](https://html.spec.whatwg.org/multipage/web-sockets.html#dom-websocket)
	///
	#[ fail( display = "An invalid url was given to the connect method: {}", _0 ) ]
	//
	InvalidUrl(String),

	/// An invalid close code was given to a close method. For valid close codes, please see:
	/// [MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/CloseEvent#Status_codes)
	///
	#[ fail( display = "An invalid close code was given to a close method: {}", _0 ) ]
	//
	InvalidCloseCode(u16),

	/// The reason string given to a close method is longer than 123 bytes, please see:
	/// [MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/close)
	///
	#[ fail( display = "The reason string given to a close method is to long." ) ]
	//
	ReasonStringToLong,

	/// Failed to connect to the server.
	///
	#[ fail( display = "Failed to connect to the server. CloseEvent: {:?}", _0 ) ]
	//
	ConnectionFailed(CloseEvent),
}



impl Fail for WsErr
{
	fn cause( &self ) -> Option< &dyn Fail >
	{
		self.inner.cause()
	}

	fn backtrace( &self ) -> Option< &Backtrace >
	{
		self.inner.backtrace()
	}
}



impl fmt::Display for WsErr
{
	fn fmt( &self, f: &mut fmt::Formatter<'_> ) -> fmt::Result
	{
		fmt::Display::fmt( &self.inner, f )
	}
}


impl WsErr
{
	/// Allows matching on the error kind
	//
	pub fn kind( &self ) -> &WsErrKind
	{
		self.inner.get_context()
	}
}

impl From<WsErrKind> for WsErr
{
	fn from( kind: WsErrKind ) -> WsErr
	{
		WsErr { inner: FailContext::new( kind ) }
	}
}

impl From< FailContext<WsErrKind> > for WsErr
{
	fn from( inner: FailContext<WsErrKind> ) -> WsErr
	{
		WsErr { inner }
	}
}
