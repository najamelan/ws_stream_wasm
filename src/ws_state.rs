use crate :: { import::*, WsErr, WsErrKind };


/// Indicates the state of a Websocket connection. The only state in which it's valid to send and receive messages
/// is OPEN.
///
/// See [MDN](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/readyState) for the ready state values.
///
#[ allow( missing_docs ) ]
//
#[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
//
pub enum WsState
{
	CONNECTING,
	OPEN      ,
	CLOSING   ,
	CLOSED    ,
}


/// Internally ready state is a u16, so it's possible to create one from a u16. Only 0-3 are valid values.
///
/// See [MDN](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/readyState) for the ready state values.
///
impl TryFrom<u16> for WsState
{
	type Error = WsErr;

	fn try_from( state: u16 ) -> Result< Self, Self::Error >
	{
		match state
		{
			0 => Ok ( WsState::CONNECTING                     ) ,
			1 => Ok ( WsState::OPEN                           ) ,
			2 => Ok ( WsState::CLOSING                        ) ,
			3 => Ok ( WsState::CLOSED                         ) ,
			_ => Err( WsErrKind::InvalidReadyState( state ).into() ) ,
		}
	}
}
