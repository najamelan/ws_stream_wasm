use crate::import::*;

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
	/// Invalid input to `WsReadyState::try_from( u16 )`
	///
	#[ fail( display = "Invalid input to conversion to WsReadyState: {}", _0 ) ]
	//
	InvalidWsState( u16 ),

	/// When trying to send and WsState is anything but WsState::Open this error is returned.
	//
	#[ fail( display = "The connection state is not \"Open\"" ) ]
	//
	ConnectionNotOpen,

	/// The port to which the connection is being attempted is being blocked.
	/// This can happen upon creating the websocket. See:
	/// [security error](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/WebSocket#Exceptions_thrown).
	/// MDN does not mention it, but normally there can also be cross domain limitations.
	///
	#[ fail( display = "The port to which the connection is being attempted is being blocked." ) ]
	//
	SecurityError,

	/// An invalid close code was given to a close method. For valid close codes, please see:
	/// [MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/CloseEvent#Status_codes)
	///
	#[ fail( display = "An invalid close code was given to a close method: {}", _0 ) ]
	//
	InvalidCloseCode(u16),

	/// The reason string given to a close method is to long, please see:
	/// [MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/close)
	///
	#[ fail( display = "The reason string given to a close method is to long." ) ]
	//
	ReasonStringToLong,
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
