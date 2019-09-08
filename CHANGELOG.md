# Changelog

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
