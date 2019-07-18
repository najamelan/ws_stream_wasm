use crate::import::*;


/// Represents a WebSocket Message, after converting from JavaScript type.
//
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



impl From< MessageEvent > for WsMessage
{
	fn from( evt: MessageEvent ) -> Self
	{
		let data = evt.data();

		if data.is_instance_of::< ArrayBuffer >()
		{
			trace!( "JsWebSocket received binary message" );

			let buf: ArrayBuffer = data.unchecked_into();

			let     buffy = Uint8Array::new( &buf );
			let mut v     = vec![ 0; buffy.length() as usize ];

			buffy.copy_to( &mut v ); // FIXME: get rid of this copy

			WsMessage::Binary( v )
		}


		else if data.is_string()
		{
			// should never fail
			//
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


impl From<WsMessage> for Vec<u8>
{
	fn from( msg: WsMessage ) -> Self
	{
		match msg
		{
			WsMessage::Text  ( string ) => string.into(),
			WsMessage::Binary( vec    ) => vec          ,
		}
	}
}
