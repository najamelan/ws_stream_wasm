# ws_stream_wasm

A convenience layer for using WebSockets from WebAssembly.

It implements a futures Stream/Sink and tokio AsyncRead/AsyncWrite on top of the web-sys interface [WebSocket](https://docs.rs/web-sys/0.3.22/web_sys/struct.WebSocket.html). This allows you to communicate between your server and a browser wasm module transparently without worrying about the underlying protocol. You can use tokio codec to get framed messages of any type that implements [serde::Serialize](https://docs.rs/serde/1.0.89/serde/trait.Serialize.html).

There are basic wrapper types around the JavaScript types MessageEvent and MessageEventData but these are not feature-complete.
This library tries to work with [async_await] wherever possible, with the exemption of WsStream because tokio is on futures 0.1. It requires a nightly compiler for now.

# Examples

For examples please run `cargo doc --open` or look at the unit tests.

# Tests

Do not forget to turn on the server:

```
cd server
cargo run
```


## TODO

In order to run the tests, go into the server crate and run `cargo run`. In another tab run `wasm-pack test  --firefox --headless`.

little note on performance:
  - `wasm-pack test  --firefox --headless --release`: 13.3s
  - `wasm-pack test  --chrome  --headless --release`: 10.4s

  - all `FIXME` in `src/` and `test/`

  - callback_future is not something that belongs in wasm_websocket_stream. Maybe a functionality like that can be added to wasm_bindgen? It should also be possible to do this for callbacks that need to take parameters.

  - the sink? It's always ready to write, it's always flushed? The websocket browser api does not really give any info about the state here... Need better unit testing to verify what happens under stress I suppose.

  - We implement AsyncRead for WsStream. This required that we implement std::io::Read, returning WouldBlock. However when calling next on the stream we get from BytesCodec, only read gets called, not poll_read. In principle tokio provides a default implementation of poll_read. It's not clear whether it would ever get called and in what circumstances. Since it is important that the current task gets woken up when a new message arrives, I have implemented poll_read so it wakes up the task. Is this necessary? It doesn't cost much to leave the implementation there now it's written.

  - We don't have Clone or Eq on JsWebSocket or WsStream... Is that ok?

