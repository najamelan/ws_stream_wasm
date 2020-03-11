# TODO

- verify example and doc tests
- design, do we want to put the use of WsIo on the user?
- text messages are accepted...

## Features
- when the connection is lost, can we know if it's the server that disconnected (correct shutdown exchange)
  or whether we have network problems.
- reconnect?
- unsafe impl Send for WsIo {}

## Testing

## Documentation
- chat client example
- automatic reconnect example using pharos to detect the close



