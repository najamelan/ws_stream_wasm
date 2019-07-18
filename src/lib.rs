//! A convenience layer for using WebSockets from WebAssembly.
//! It implements a futures Stream/Sink and tokio AsyncRead/AsyncWrite on top of the web-sys interface
//! [WebSocket](https://docs.rs/web-sys/0.3.17/web_sys/struct.WebSocket.html).
//!
//! This allows you to communicate between your server and a browser wasm module transparently without worrying about
//! the underlying protocol. You can use tokio codec to get framed messages of any type that implements [serde::Serialize](https://docs.rs/serde/1.0.89/serde/trait.Serialize.html).
//!
//! This library tries to work with [async_await] wherever possible, with the exemption of WsStream because tokio is on futures 0.1.
//! It requires a nightly compiler for now.
//!
//! For examples please have a look at [JsWebSocket] and [WsStream].
//!
#![ doc    ( html_root_url = "https://docs.rs/ws_stream_wasm" ) ]
#![ feature( async_await                                      ) ]
#![ deny   ( missing_docs                                     ) ]
#![ forbid ( unsafe_code                                      ) ]
#![ allow  ( clippy::suspicious_else_formatting               ) ]

mod error           ;
mod ws_message      ;
mod ws_io           ;
mod ws_state        ;
mod ws_stream       ;
mod callback_future ;

pub use
{
	ws_state          :: { WsState                } ,
	callback_future   :: { future_event           } ,
	error             :: { WsErr      , WsErrKind } ,
	ws_message        :: { WsMessage              } ,
	ws_io             :: { WsIo                   } ,
	ws_stream         :: { WsStream               } ,
};



mod import
{
	pub(crate) use
	{
		async_runtime :: { rt                                                                      } ,
		failure       :: { Backtrace, Fail, Context as FailContext                                 } ,
		futures       :: { channel::mpsc::unbounded, Poll                                          } ,
		futures       :: { prelude::{ Stream, Sink, AsyncWrite, AsyncRead }, stream::{ StreamExt } } ,
		futures       :: { ready                                                                   } ,
		std           :: { io, cmp, collections::VecDeque, fmt, task::{ Context, Waker }           } ,
		std           :: { rc::Rc, cell::{ RefCell }, pin::Pin, convert::{ TryFrom, TryInto }      } ,
		log           :: { *                                                                       } ,
		js_sys        :: { ArrayBuffer, Uint8Array                                                 } ,
		wasm_bindgen  :: { closure::Closure, JsCast, JsValue, UnwrapThrowExt                       } ,
		web_sys       :: { *, BinaryType, Blob, WebSocket                                          } ,
		js_sys        :: { Array                                                                   } ,
	};
}
