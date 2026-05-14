//! Audiobookshelf-compatible socket.io payload + emitter layer.
//!
//! 100 % byte-shape compatible with the upstream events documented in
//! `server/SocketAuthority.js` and `server/managers/PlaybackSessionManager.js`.
pub mod broadcaster;
pub mod events;
pub mod gateway;
