//! Low-level implementation of the connection-upgrade process
//! on the basis of the [`libp2p` connections][libp2p-conn-spec] specification.
//!
//! [libp2p-conn-spec]: https://github.com/libp2p/specs/blob/master/connections/README.md
pub mod connection;
pub mod multistream_select;
pub mod noise;
