//! Crate specific errors.
//
use crate::{ import::*, CloseEvent };


/// The error type for errors happening in `ws_stream_wasm`.
///
/// Use [`Error::kind()`] to know which kind of error happened.
//
#[ derive( Debug, Clone, PartialEq, Eq ) ]
//
pub struct Error
{
	pub(crate) kind: ErrorKind
}



/// The different kind of errors that can happen when you use the `ws_stream` API.
//
#[ derive( Debug, Clone, PartialEq, Eq ) ]
//
pub enum ErrorKind
{
	/// Invalid input to [WsState::try_from( u16 )](crate::WsState)
	//
	InvalidWsState
	{
		/// The user supplied value that is in valid
		//
		supplied: u16
	},

	/// When trying to send and [WsState](crate::WsState) is anything but [WsState::Open](crate::WsState::Open) this error is returned.	//
	ConnectionNotOpen,

	/// Browsers will forbid making websocket connections to certain ports. See this [Stack Overflow question](https://stackoverflow.com/questions/4313403/why-do-browsers-block-some-ports/4314070).
	//
	ForbiddenPort,

	/// An invalid URL was given to [WsStream::connect](crate::WsStream::connect), please see:
	/// [HTML Living Standard](https://html.spec.whatwg.org/multipage/web-sockets.html#dom-websocket)
	//
	InvalidUrl
	{
		/// The user supplied value that is in valid
		//
		supplied: String
	},

	/// An invalid close code was given to a close method. For valid close codes, please see:
	/// [MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/CloseEvent#Status_codes)
	//
	InvalidCloseCode
	{
		/// The user supplied value that is in valid
		//
		supplied: u16
	},


	/// The reason string given to a close method is longer than 123 bytes, please see:
	/// [MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/close)
	//
	ReasonStringToLong,


	/// Failed to connect to the server.
	//
	ConnectionFailed
	{
		/// The close event that might hold extra code and reason information.
		//
		event: CloseEvent
	},


	#[ doc( hidden ) ]
	//
	__NonExhaustive__
}



impl ErrorTrait for Error {}



impl fmt::Display for ErrorKind
{
	fn fmt( &self, f: &mut fmt::Formatter<'_> ) -> fmt::Result
	{
		match self
		{
			Self::InvalidWsState{ supplied } =>

				write!( f, "Invalid input to conversion to WsReadyState: {}", supplied ),


			Self::ConnectionNotOpen => write!( f, "The connection state is not \"Open\"." ),


			Self::ForbiddenPort =>

				write!( f, "The port to which the connection is being attempted is not allowed." ),


			Self::InvalidUrl{ supplied } =>

				write!( f, "An invalid URL was given to the connect method: {}", supplied ),


			Self::InvalidCloseCode{ supplied } =>

				write!( f, "An invalid close code was given to a close method: {}", supplied ),


			Self::ReasonStringToLong =>

				write!( f, "The reason string given to a close method is to long." ),


			Self::ConnectionFailed{ event } =>

				write!( f, "Failed to connect to the server. CloseEvent: {:?}", event ),


			Self::__NonExhaustive__ => unreachable!(),
		}
	}
}


impl fmt::Display for Error
{
	fn fmt( &self, f: &mut fmt::Formatter<'_> ) -> fmt::Result
	{
		write!( f, "ws_stream::Error: {}", self.kind )
	}
}



impl Error
{
	/// Allows matching on the error kind
	//
	pub fn kind( &self ) -> &ErrorKind
	{
		&self.kind
	}
}

impl From<ErrorKind> for Error
{
	fn from( kind: ErrorKind ) -> Error
	{
		Error { kind }
	}
}



