// See: https://github.com/rust-lang/rust/issues/44732#issuecomment-488766871
//
#![cfg_attr( feature = "docs", feature(doc_cfg, external_doc) )]
#![cfg_attr( feature = "docs", doc(include = "../README.md")  )]
//!
#![ doc    ( html_root_url = "https://docs.rs/ws_stream_wasm"            ) ]
#![ deny   ( missing_docs                                                ) ]
#![ forbid ( unsafe_code                                                 ) ]
#![ allow  ( clippy::suspicious_else_formatting, clippy::needless_return ) ]


#![ warn
(
	missing_debug_implementations ,
	nonstandard_style             ,
	rust_2018_idioms              ,
	trivial_casts                 ,
	trivial_numeric_casts         ,
	unused_extern_crates          ,
	unused_qualifications         ,
	single_use_lifetimes          ,
	unreachable_pub               ,
	variant_size_differences      ,
)]



pub mod error       ;
    mod ws_event    ;
    mod ws_message  ;
    mod ws_io       ;
    mod ws_state    ;
    mod ws_meta     ;

pub use
{
	error      :: { Error as WsErr, ErrorKind as WsErrKind } ,
	ws_event   :: { WsEvent, CloseEvent                    } ,
	ws_message :: { WsMessage                              } ,
	ws_io      :: { WsIo                                   } ,
	ws_meta    :: { WsMeta                                 } ,
	ws_state   :: { WsState                                } ,
};



mod import
{
	pub(crate) use
	{
		futures              :: { prelude::{ Stream, Sink, AsyncWrite, AsyncRead }, ready                } ,
		futures              :: { StreamExt, SinkExt                                                     } ,
		std                  :: { io, cmp, collections::VecDeque, fmt, task::{ Context, Waker, Poll }    } ,
		std                  :: { rc::Rc, cell::{ RefCell }, pin::Pin, convert::{ TryFrom, TryInto }     } ,
		std                  :: { error::Error as ErrorTrait                                             } ,
		log                  :: { *                                                                      } ,
		js_sys               :: { ArrayBuffer, Uint8Array                                                } ,
		wasm_bindgen         :: { closure::Closure, JsCast, JsValue, UnwrapThrowExt                      } ,
		web_sys              :: { *, BinaryType, Blob, WebSocket, CloseEvent as JsCloseEvt, DomException } ,
		js_sys               :: { Array                                                                  } ,
		pharos               :: { Pharos, Observable, Filter, ObserveConfig, Events                      } ,
		wasm_bindgen_futures :: { spawn_local                                                            } ,
		ws_stream_io         :: { WsIo as WsAsync                                                        } ,
	};

	#[ cfg( feature = "tokio_io" ) ]
	//
	pub(crate) use tokio::io::{ AsyncRead as TokAsyncRead, AsyncWrite as TokAsyncWrite };
}


use import::*;

/// Helper function to reduce code bloat
//
pub(crate) fn notify( pharos: Rc<RefCell<Pharos<WsEvent>>>, evt: WsEvent )
{
	let notify = async move
	{
		let mut pharos = pharos.borrow_mut();

		pharos.send( evt ).await

			.map_err( |e| unreachable!( "{:?}", e ) ).unwrap(); // only happens if we closed it.
	};

	spawn_local( notify );
}
