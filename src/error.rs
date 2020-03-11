//! Crate specific errors.
//
use crate::{ import::*, CloseEvent };


/// The error type for errors happening in `ws_stream_wasm`.
///
/// Use [`Error::kind()`] to know which kind of error happened.
//
#[ derive( Debug, Error, Clone, PartialEq, Eq ) ] #[ non_exhaustive ]
//
pub enum WsErr
{
	/// Invalid input to [WsState::try_from( u16 )](crate::WsState)
	//
	#[ error( "Invalid input to conversion to WsReadyState: {supplied}" ) ]
	//
	InvalidWsState
	{
		/// The user supplied value that is in valid
		//
		supplied: u16
	},

	/// When trying to send and [WsState](crate::WsState) is anything but [WsState::Open](crate::WsState::Open) this error is returned.
	//
	#[ error( "The connection state is not \"Open\"." ) ]
	//
	ConnectionNotOpen,

	/// Browsers will forbid making websocket connections to certain ports. See this [Stack Overflow question](https://stackoverflow.com/questions/4313403/why-do-browsers-block-some-ports/4314070).
	//
	#[ error( "The port to which the connection is being attempted is not allowed." ) ]
	//
	ForbiddenPort,

	/// An invalid URL was given to [WsMeta::connect](crate::WsMeta::connect), please see:
	/// [HTML Living Standard](https://html.spec.whatwg.org/multipage/web-sockets.html#dom-websocket)
	//
	#[ error( "An invalid URL was given to the connect method: {supplied}" ) ]
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
	#[ error( "An invalid close code was given to a close method: {supplied}" ) ]
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
	#[ error( "The reason string given to a close method is to long." ) ]
	//
	ReasonStringToLong,


	/// Failed to connect to the server.
	//
	#[ error( "Failed to connect to the server. CloseEvent: {event:?}" ) ]
	//
	ConnectionFailed
	{
		/// The close event that might hold extra code and reason information.
		//
		event: CloseEvent
	},
}


