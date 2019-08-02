# Changelog

## 0.2 - 2019-08-02

- **BREAKING CHANGE**: Fix: Correctly wake up tasks waiting for a next message if the connection gets closed externally.
  This prevents these tasks from hanging indefinitely.
  As a consequence, `WsStream::close` now returns a `Result`, taking into account that if the connection is already
  closed, we don't have the `CloseEvent`. Instead a `WsErr` of kind `WsErrKind::ConnectionNotOpen` is returned.
- update to async_runtime 0.3
