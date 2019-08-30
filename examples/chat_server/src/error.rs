use crate::{ import::* };

/// The error type for errors happening in `ws_stream_wasm`.
///
/// Use [`ChatErr::kind()`] to know which kind of error happened.
//
#[ derive( Debug ) ]
//
pub struct ChatErr
{
	inner: FailContext<ChatErrKind>,
}



/// The different kind of errors that can happen when you use the `ws_stream_wasm` API.
//
#[ derive( Clone, PartialEq, Eq, Debug, Fail ) ]
//
pub enum ChatErrKind
{
	/// Invalid nick
	///
	#[ fail( display = "The nick you specify must be between 1 and 15 word characters, was invalid: '{}'.", _0 ) ]
	//
	NickInvalid( String ),

	/// Nick in use
	///
	#[ fail( display = "The nick you chose is already in use: '{}'.", _0 ) ]
	//
	NickInUse( String ),

	/// Nick in use
	///
	#[ fail( display = "The new nick you chose is the same as your old one: '{}'.", _0 ) ]
	//
	NickUnchanged( String ),
}



impl Fail for ChatErr
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



impl fmt::Display for ChatErr
{
	fn fmt( &self, f: &mut fmt::Formatter<'_> ) -> fmt::Result
	{
		fmt::Display::fmt( &self.inner, f )
	}
}


impl ChatErr
{
	/// Allows matching on the error kind
	//
	pub fn kind( &self ) -> &ChatErrKind
	{
		self.inner.get_context()
	}
}

impl From<ChatErrKind> for ChatErr
{
	fn from( kind: ChatErrKind ) -> ChatErr
	{
		ChatErr { inner: FailContext::new( kind ) }
	}
}

impl From< FailContext<ChatErrKind> > for ChatErr
{
	fn from( inner: FailContext<ChatErrKind> ) -> ChatErr
	{
		ChatErr { inner }
	}
}
