# Changelog

## 0.4.0 - 2019-09-23

  - **BREAKING CHANGE**: use the new filter feature from pharos, making `NextEvent` and `WsEventType` redundant. Those
    types have been removed from the library. The `observe` and method off `WsStream` now takes a `pharos::ObserveConfig` to filter event types. Please refer to the documentation of [pharos](https://docs.rs/pharos) for how to use them.

## 0.3.0 - 2019-09-08

  - drop dependencies on async_runtime and failure and switch to std::error::Error for error handling
  - add a fullstack chat example (still needs documentation and cleanup)

## 0.2.1 - 2019-08-02

  - Fix incorrect link to changelog in readme


## 0.2.0 - 2019-08-02

  - **BREAKING CHANGE**: Fix: Correctly wake up tasks waiting for a next message if the connection gets closed externally.
    This prevents these tasks from hanging indefinitely.
    As a consequence, `WsStream::close` now returns a `Result`, taking into account that if the connection is already
    closed, we don't have the `CloseEvent`. Instead a `WsErr` of kind `WsErrKind::ConnectionNotOpen` is returned.
  - update to async_runtime 0.3
