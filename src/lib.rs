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
#![ doc    ( html_root_url = "https://docs.rs/wasm_websocket_stream/0.1.0" ) ]
#![ feature( async_await                                                   ) ]
#![ deny   ( missing_docs                                                  ) ]
#![ forbid ( unsafe_code                                                   ) ]
#![ allow  ( clippy::suspicious_else_formatting                            ) ]

mod error             ;
mod js_msg_event      ;
mod ws_io             ;
mod ws_state          ;
mod ws_io_binary      ;
mod ws_stream         ;
mod callback_future   ;

pub use
{
	ws_state          :: { WsState                    } ,
	callback_future   :: { future_event               } ,
	error             :: { WsErr      , WsErrKind     } ,
	js_msg_event      :: { JsMsgEvent , WsMessage  } ,
	ws_io             :: { WsIo                       } ,
	ws_io_binary      :: { WsIoBinary                 } ,
	ws_stream         :: { WsStream                   } ,
};



mod import
{
	pub(crate) use
	{
		failure      :: { Backtrace, Fail, Context as FailContext                            } ,
		futures      :: { channel::{ mpsc::unbounded }, Poll                                 } ,
		futures      :: { prelude::{ Stream, Sink, AsyncWrite }, stream::{ StreamExt }       } ,
		futures      :: { task::Context, ready                                               } ,
		std          :: { io::{ self }, collections::VecDeque, fmt, task::Waker              } ,
		std          :: { rc::Rc, cell::{ RefCell }, pin::Pin, convert::{ TryFrom, TryInto } } ,
		log          :: { *                                                                  } ,
		js_sys       :: { ArrayBuffer, Uint8Array                                            } ,
		wasm_bindgen :: { closure::Closure, JsCast, JsValue, UnwrapThrowExt                  } ,
		web_sys      :: { *, console::debug_1 as dbg, BinaryType, Blob, WebSocket            } ,
	};
}
