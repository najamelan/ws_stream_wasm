use crate::import::*;


/// A wrapper around the [`web_sys::MessageEvent`](https://docs.rs/web-sys/0.3.17/web_sys/struct.MessageEvent.html) for convenience.
/// Allows to extract data as a [WsMessage].
///
#[ derive( Debug, Clone ) ]
//
pub struct JsMsgEvent
{
	/// The wrapped web_sys::MessageEvent if you need it
	///
	pub msg_evt: MessageEvent
}


impl JsMsgEvent
{
	/// The data contained by the message.
	///
	pub fn data( &self ) -> WsMessage
	{
		WsMessage::from( self )
	}
}


/// [Data](https://docs.rs/web-sys/0.3.17/web_sys/struct.MessageEvent.html#method.data) contained in a MessageEvent. See:
/// [Html5 specs](https://html.spec.whatwg.org/multipage/web-sockets.html#feedback-from-the-protocol)
///
/// Data can be a string or binary.
///
#[ derive( Debug, Clone, PartialEq, Eq, Hash ) ]
//
pub enum WsMessage
{
	/// The data of the message is a string
	///
	Text  ( String  ),

	/// The message contains binary data
	///
	Binary( Vec<u8> ),
}



impl From< &JsMsgEvent > for WsMessage
{
	fn from( evt: &JsMsgEvent ) -> Self
	{
		let data = evt.msg_evt.data();

		if data.is_instance_of::< ArrayBuffer >()
		{
			trace!( "JsWebSocket received binary message" );

			let buf = data.dyn_into::< ArrayBuffer >().unwrap_throw();

			let     buffy          = Uint8Array::new( buf.as_ref() );
			let mut v    : Vec<u8> = vec![ 0; buffy.length() as usize ];

			buffy.copy_to( &mut v ); // FIXME: get rid of this copy

			WsMessage::Binary( v )
		}


		else if data.is_string()
		{
			let text = data.as_string().expect_throw( "From< &JsMsgEvent > for WsMessage: data.as_string()" );

			WsMessage::Text( text )
		}


		// We have set the binary mode to array buffer, so normally this shouldn't happen. That is as long
		// as this is used within the context of the WsStream constructor.
		//
		// FIXME: find a way to convert a blob...
		//
		else if data.is_instance_of::< Blob >()
		{
			error!( "JsWebSocket received a blob...unimplemented!" );

			unreachable!();
		}


		else
		{
			error!( "JsWebSocket received data that is not string, nor ArrayBuffer, nor Blob, bailing..." );

			unreachable!();
		}
	}
}
