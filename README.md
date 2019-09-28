# ws_stream_wasm

[![standard-readme compliant](https://img.shields.io/badge/readme%20style-standard-brightgreen.svg?style=flat-square)](https://github.com/RichardLitt/standard-readme)
[![Build Status](https://api.travis-ci.org/najamelan/ws_stream_wasm.svg?branch=master)](https://travis-ci.org/najamelan/ws_stream_wasm)
[![Docs](https://docs.rs/ws_stream_wasm/badge.svg)](https://docs.rs/ws_stream_wasm)
[![crates.io](https://img.shields.io/crates/v/ws_stream_wasm.svg)](https://crates.io/crates/ws_stream_wasm)


> A convenience library for using web sockets in WASM

**features:**
- [`WsStream`]: A wrapper around [`web_sys::WebSocket`](https://docs.rs/web-sys/0.3.27/web_sys/struct.WebSocket.html).
- [`WsMessage`]: A simple rusty representation of a WebSocket message.
- [`WsIo`]: A futures Sink/Stream of WsMessage. (can use the futures compat layer to get futures 01 versions).
                It also implements AsyncRead/AsyncWrite from futures 0.3. With the compat layer you can obtain futures
                01 versions for use with tokio codec.
- [`WsEvent`]: [`WsStream`] is observable with [pharos](https://crates.io/crates/pharos) for events (mainly useful for connection close).

**NOTE:** this crate only works on WASM. If you want a server side equivalent that implements AsyncRead/AsyncWrite over
WebSockets, check out [ws_stream_tungstenite](https://crates.io/crates/ws_stream_tungstenite).

**missing features:**
- no automatic reconnect
- not all features are thoroughly tested. Notably, I have little use for extensions and sub-protocols. Tungstenite,
  which I use for the server end (and for automated testing) doesn't support these, making it hard to write unit tests.

## Table of Contents

- [Install](#install)
  - [Upgrade](#upgrade)
  - [Dependencies](#dependencies)
- [Usage](#usage)
  - [API](#api)
- [References](#references)
- [Contributing](#contributing)
  - [Code of Conduct](#code-of-conduct)
- [License](#license)


## Install
With [cargo add](https://github.com/killercup/cargo-edit):
`cargo add ws_stream_wasm`

With [cargo yaml](https://gitlab.com/storedbox/cargo-yaml):
```yaml
dependencies:

  ws_stream_wasm: ^0.5
```

With raw Cargo.toml
```toml
[dependencies]

   ws_stream_wasm = "0.5"
```

### Upgrade

Please check out the [changelog](https://github.com/najamelan/ws_stream_wasm/blob/master/CHANGELOG.md) when upgrading.

### Dependencies

This crate has few dependencies. Cargo will automatically handle it's dependencies for you.

There are no optional features.


## Usage

Please have a look in the [examples directory of the repository](https://github.com/najamelan/ws_stream_wasm/tree/master/examples).

The [integration tests](https://github.com/najamelan/ws_stream_wasm/tree/master/tests) are also useful.

### Basic events example
```rust
use
{
   ws_stream_wasm       :: *                        ,
   pharos               :: *                        ,
   wasm_bindgen         :: UnwrapThrowExt           ,
   wasm_bindgen_futures :: futures_0_3::spawn_local ,
   futures              :: stream::StreamExt        ,
};

let program = async
{
   let (mut ws, _wsio) = WsStream::connect( "127.0.0.1:3012", None ).await

      .expect_throw( "assume the connection succeeds" );

   let mut evts = ws.observe( ObserveConfig::default() ).expect_throw( "observe" );

   ws.close().await;

   // Note that since WsStream::connect resolves to an opened connection, we don't see
   // any Open events here.
   //
   assert!( evts.next().await.unwrap_throw().is_closing() );
   assert!( evts.next().await.unwrap_throw().is_closed () );
};

spawn_local( program );
```

### Filter events example

This shows how to filter events. The functionality comes from the pharos crate which we use to make
[`WsStream`] observable.

```rust
use
{
   ws_stream_wasm       :: *                        ,
   pharos               :: *                        ,
   wasm_bindgen         :: UnwrapThrowExt           ,
   wasm_bindgen_futures :: futures_0_3::spawn_local ,
   futures              :: stream::StreamExt        ,
};

let program = async
{
   let (mut ws, _wsio) = WsStream::connect( "127.0.0.1:3012", None ).await

      .expect_throw( "assume the connection succeeds" );

   // The Filter type comes from the pharos crate.
   //
   let mut evts = ws.observe( Filter::Pointer( WsEvent::is_closed ).into() ).expect_throw( "observe" );

   ws.close().await;

   // Note we will only get the closed event here, the WsEvent::Closing has been filtered out.
   //
   assert!( evts.next().await.unwrap_throw().is_closed () );
};

spawn_local( program );
```

## API

Api documentation can be found on [docs.rs](https://docs.rs/ws_stream_wasm).


## References
The reference documents for understanding web sockets and how the browser handles them are:
- [HTML Living Standard](https://html.spec.whatwg.org/multipage/web-sockets.html)
- [RFC 6455 - The WebSocket Protocol](https://tools.ietf.org/html/rfc6455)


## Contributing

This repository accepts contributions. Ideas, questions, feature requests and bug reports can be filed through Github issues.

Pull Requests are welcome on Github. By committing pull requests, you accept that your code might be modified and reformatted to fit the project coding style or to improve the implementation. Please discuss what you want to see modified before filing a pull request if you don't want to be doing work that might be rejected.

Please file PR's against the `dev` branch, don't forget to update the changelog and the documentation.

### Testing

For testing we need back-end servers to echo data back to the tests. These are in the `ws_stream_tungstenite` crate.
```shell
git clone https://github.com/najamelan/ws_stream_tungstenite
cd ws_stream_tungstenite
cargo run --example echo --release

# in a different terminal:
cargo run --example echo_tt --release -- "127.0.0.1:3312"

# the second server is pure tokio-tungstenite without ws_stream wrapping it in AsyncRead/Write. This
# is needed for testing a WsMessage::Text because ws_stream only does binary.

# in a third terminal, in ws_stream_wasm you have different options:
wasm-pack test --firefox [--headless] [--release]
wasm-pack test --chrome  [--headless] [--release]
```

In general chrome is well faster. When running it in the browser (without `--headless`) you get trace logging
in the console, which helps debugging. In chrome you need to enable verbose output in the console,
otherwise only info and up level are reported.

### Code of conduct

Any of the behaviors described in [point 4 "Unacceptable Behavior" of the Citizens Code of Conduct](http://citizencodeofconduct.org/#unacceptable-behavior) are not welcome here and might get you banned. If anyone including maintainers and moderators of the project fail to respect these/your limits, you are entitled to call them out.

## License

[Unlicence](https://unlicense.org/)

