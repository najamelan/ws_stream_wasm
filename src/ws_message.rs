use crate::{ import::*, WsErr };


/// Represents a WebSocket Message, after converting from JavaScript type.
//
#[ derive( Debug, Clone, PartialEq, Eq, Hash ) ]
//
pub enum WsMessage
{
	/// The data of the message is a string.
	///
	Text  ( String  ),

	/// The message contains binary data.
	///
	Binary( Vec<u8> ),
}



/// This will convert the JavaScript event into a WsMessage. Note that this
/// will only work if the connection is set to use the binary type ArrayBuffer.
/// On binary type Blob, this will panic.
//
impl TryFrom< MessageEvent > for WsMessage
{
	type Error = WsErr;

	fn try_from( evt: MessageEvent ) -> Result< Self, Self::Error >
	{
		match evt.data()
		{
			d if d.is_instance_of::< ArrayBuffer >() =>
			{
				let     buffy = Uint8Array::new( d.unchecked_ref() );
				let mut v     = vec![ 0; buffy.length() as usize ];

				buffy.copy_to( &mut v ); // FIXME: get rid of this copy

				Ok( WsMessage::Binary( v ) )
			}


			d if d.is_string() =>
			{
				match d.as_string()
				{
					Some(text) => Ok ( WsMessage::Text( text ) ),
					None       => Err( WsErr::InvalidEncoding  ),
				}
			}


			// We have set the binary mode to array buffer (WsMeta::connect), so normally this shouldn't happen.
			// That is as long as this is used within the context of the WsMeta constructor.
			//
			d if d.is_instance_of::< Blob >() =>
			{
				error!( "WsMessage::try_from received a blob...cannot convert to WsMessage" );

				Err( WsErr::CantDecodeBlob )
			}


			_ =>
			{
				error!( "WsMessage::try_from received data that is not String, nor ArrayBuffer, nor Blob, bailing..." );

				Err( WsErr::UnknownDataType )
			}
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


impl From<Vec<u8>> for WsMessage
{
	fn from( vec: Vec<u8> ) -> Self
	{
		WsMessage::Binary( vec )
	}
}


impl From<String> for WsMessage
{
	fn from( s: String ) -> Self
	{
		WsMessage::Text( s )
	}
}


impl AsRef<[u8]> for WsMessage
{
	fn as_ref( &self ) -> &[u8]
	{
		match self
		{
			WsMessage::Text  ( string ) => string.as_ref() ,
			WsMessage::Binary( vec    ) => vec   .as_ref() ,
		}
	}
}



