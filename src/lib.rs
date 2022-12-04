//! An elementary light-client library for handshake-ready nodes in substrate-based p2p networks.
//!
//! There are two modules exposed:
//!
//! * [`scratch`][]: A low-level implementation of the handshake.
//! * [`swarm`][]: A high-level implementation using `libp2p-swarm` API.
pub mod error;
pub mod scratch;
pub mod swarm;
