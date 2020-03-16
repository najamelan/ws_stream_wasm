# TODO

- why are the tests for futures_codec so much faster than for tokio codec.
- verify example and doc tests
- look into proper changelogs, like the futures crate.
- design, do we want to put the use of WsIo on the user?
- text messages are accepted...
- update tokio-util, breaking change, probably needs new version of tokio-serde-cbor
- ci: https://rustwasm.github.io/docs/wasm-bindgen/wasm-bindgen-test/continuous-integration.html has some windows instructions.

## Features
- when the connection is lost, can we know if it's the server that disconnected (correct shutdown exchange)
  or whether we have network problems.
- reconnect?

## Testing

## Documentation
- chat client example
- automatic reconnect example using pharos to detect the close



