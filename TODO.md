# TODO

## Features
- include events (pharos)
- reconnect?

- derive Debug/Clone/PartialEq/Eq on all types
- remove JsMsgEvent and just keep WsMessage?
- implement AsyncRead ourselves and not wait for futures to merge our pull request.
  this has the added benefit that we can have a stream of Vec<u8> and not of IoResult.

## Testing
- what if server refuses connection? What error?
- check all TODO/FIXME
- check all unwrap/expect
- verify all features are tested
- verify Cargo.yml + all dependencies
- mention above all integration tests what is tested
- enable travis CI

## Documentation
- chat client example
- documentation
  - copy readme to lib.rs + (attributes)
