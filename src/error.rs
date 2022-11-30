//! Library specific errors
use libp2p::{core::transport, multiaddr, swarm::DialError};
use std::net::AddrParseError;
use thiserror::Error;

/// Variants of service-specific errors.
#[derive(Error, Debug)]
pub enum PadawanError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    AddressParse(#[from] AddrParseError),
    #[error(transparent)]
    SwarmDial(#[from] DialError),
    #[error(transparent)]
    Transport(#[from] transport::TransportError<std::io::Error>),
    #[error(transparent)]
    Multiaddr(#[from] multiaddr::Error),
}

/// Alias for a `std::result::Result` that always return an error of type [`PadawanError`][].
pub type Result<T> = std::result::Result<T, PadawanError>;
