# ws_stream_wasm

[![standard-readme compliant](https://img.shields.io/badge/readme%20style-standard-brightgreen.svg?style=flat-square)](https://github.com/RichardLitt/standard-readme)
[![Build Status](https://api.travis-ci.org/najamelan/ws_stream_wasm.svg?branch=master)](https://travis-ci.org/najamelan/ws_stream_wasm)
[![Docs](https://docs.rs/ws_stream_wasm/badge.svg)](https://docs.rs/ws_stream_wasm)
[![crates.io](https://img.shields.io/crates/v/ws_stream_wasm.svg)](https://crates.io/crates/ws_stream_wasm)


> A convenience library for using websockets in WASM

**features:**
- `WsStream`  : A wrapper around [`web_sys::WebSocket`](https://docs.rs/web-sys/0.3.25/web_sys/struct.WebSocket.html).
- `WsMessage` : A simple rusty representation of a WebSocket message.
- `WsIo`      : A futures Sink/Stream of WsMessage. (can use the futures compat layer to get futures 01 versions).
- `WsIoBinary`: A futures Sink/Stream of Vec<u8>, implements AsyncWrite. You can obtain an object that implements
	AsyncRead and AsyncWrite by calling `.into_async_read()` from the futures `TryStreamExt` trait. With the compat
	layer you can obtain futures 01 versions for use with tokio.

**NOTE:** this crate only works on WASM. If you want a server side equivalent that implements AsyncRead/AsyncWrite over
WebSockets, check out [ws_stream](https://crates.io/crates/ws_stream).

**missing features:**
- no automatic reconnect
- no events (probably I'll make it Observable with [pharos](https://crates.io/crates/pharos) one day)
- not all features are thoroughly tested. Notably, I have little use for extensions and subprotocols. Tungstenite,
  which I use for the server end (and for automated testing) doesn't support these, making it hard to write unit tests.

## Table of Contents

- [Install](#install)
  - [Dependencies](#dependencies)
- [Usage](#usage)
- [API](#api)
- [Contributing](#contributing)
  - [Code of Conduct](#code-of-conduct)
- [License](#license)


## Install
With [cargo add](https://github.com/killercup/cargo-edit):
`cargo add ws_stream_wasm`

With [cargo yaml](https://gitlab.com/storedbox/cargo-yaml):
```yaml
dependencies:

  ws_stream_wasm: ^0.1
```

With raw Cargo.toml
```toml
[dependencies]

   ws_stream_wasm = "^0.1"
```

### Dependencies

This crate has few dependiencies. Cargo will automatically handle it's dependencies for you.

There are no optional features.

## Usage

Please have a look in the [examples directory of the repository](https://github.com/najamelan/ws_stream_wasm/tree/master/examples).

The [integration tests](https://github.com/najamelan/ws_stream_wasm/tree/master/tests) are also useful.

## API

Api documentation can be found on [docs.rs](https://docs.rs/ws_stream_wasm).


## Contributing

This repository accepts contributions. Ideas, questions, feature requests and bug reports can be filed through github issues.

Pull Requests are welcome on github. By commiting pull requests, you accept that your code might be modified and reformatted to fit the project coding style or to improve the implementation. Please discuss what you want to see modified before filing a pull request if you don't want to be doing work that might be rejected.


### Code of conduct

Any of the behaviors described in [point 4 "Unacceptable Behavior" of the Citizens Code of Conduct](http://citizencodeofconduct.org/#unacceptable-behavior) are not welcome here and might get you banned. If anyone including maintainers and moderators of the project fail to respect these/your limits, you are entitled to call them out.

## License

[Unlicence](https://unlicense.org/)

