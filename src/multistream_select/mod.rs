//! Implement encoding and transport according to the
//! [`multistream_select`][multistream_select]
//! specification.
//!
//! [multistream_select]: https://github.com/multiformats/multistream-select

use unsigned_varint as varint;

use crate::error::PadawanError;

pub mod mirror;

const MULTISTREAM: &[u8] = b"/multistream/1.0.0\n";
const NOISE: &[u8] = b"/noise\n";
const YAMUX: &[u8] = b"/yamux/1.0.0\n";
const NA: &[u8] = b"na\n";

/// Encode an arbitrary slice of bytes according to the `multistream_select` specification
pub fn encode(bytes: &[u8]) -> Vec<u8> {
    let mut varint = [0; 10];
    let mut encoded = Vec::from(varint::encode::usize(bytes.len(), &mut varint));
    encoded.extend_from_slice(bytes);
    encoded
}

/// Supported protocols
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Protocol {
    Multistream,
    Noise,
    Yamux,
    NotAvailable,
}

impl Protocol {
    /// The names of the protocol as bytes
    pub fn name(&self) -> &'static [u8] {
        match self {
            Self::Multistream => MULTISTREAM,
            Self::Noise => NOISE,
            Self::Yamux => YAMUX,
            Self::NotAvailable => NA,
        }
    }

    /// Decode a byte slice into a protocol
    ///
    /// # Errors
    ///
    /// Fails if there is a mismatch between the `varint` prefix
    /// and the protocol length, or in case of `varint` decode
    /// errors.
    pub fn decode(encoded: &[u8]) -> Result<Self, PadawanError> {
        let (len, protocol) = varint::decode::usize(encoded)?;
        if len != protocol.len() {
            return Err(PadawanError::InvalidMultistreamEncoding);
        }
        Ok(match protocol {
            MULTISTREAM => Self::Multistream,
            NOISE => Self::Noise,
            YAMUX => Self::Yamux,
            _ => Self::NotAvailable,
        })
    }

    /// Encode the protocol
    pub fn encode(self) -> Vec<u8> {
        encode(self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protocol_decode_supported() {
        let protocol = Protocol::Yamux;
        assert!(matches!(
            Protocol::decode(&protocol.encode()),
            Ok(Protocol::Yamux)
        ));
    }

    #[test]
    fn protocol_decode_unsupported() {
        let protocol = b"unsupported";
        assert!(matches!(
            Protocol::decode(&encode(protocol)),
            Ok(Protocol::NotAvailable)
        ));
    }

    #[test]
    fn protocol_decode_invalid() {
        let protocol = b"invalid";
        assert!(Protocol::decode(protocol).is_err());
    }
}
