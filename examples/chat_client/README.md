# Ws_stream_wasm chat client example

Demonstration of `ws_stream_wasm` working in WASM. This example shows a rather realistic (error handling, security, basic features) chat client that communicates with a chat server over websockets. The communication with the server happens with
a custom enum, serialized with a cbor codec (for futures-codec, not tokio), over AsyncRead/AsyncWrite 0.3.

What ws_stream_wasm adds here is that we just frame the connection with a codec instead of manually serializing our
data structure, creating a websocket message with `web_sys`, and deal with all the potential errors on the connection
by hand.

## Install

This requires you to run the chat_server example from [ws_stream](https://github.com/najamelan/ws_stream). You can tweak
the ip:port to something else if you want (for the server you can pass it on the cmd line, for the client it's hard coded).

You will need wasm-pack:
```bash
cargo install wasm-pack

# and compile the client
#
wasm-pack build --release --target web

# in ws_stream repo
# make sure this is running in the same network namespace as your browser
#
cargo run --example chat_server --release
```

## Usage

Now you can open the `index.html` from this crate in several web browser tabs and chat away.


## TODO
- disconnect button
- reread all code and cleanup
- document as example
- gui
- blog post?
