//! Implement encoding and transport according to the
//! [`multistream_select`][multistream_select]
//! specification.
//!
//! [multistream_select]: https://github.com/multiformats/multistream-select

use unsigned_varint::encode;

pub mod mirror;

const MULTISTREAM: &[u8] = b"/multistream/1.0.0\n";
const NOISE: &[u8] = b"/noise\n";
const YAMUX: &[u8] = b"/yamux/1.0.0\n";

/// Supported protocols
#[derive(Debug, Clone, Copy, Hash)]
pub enum Protocol {
    Multistream,
    Noise,
    Yamux,
}

impl Protocol {
    pub fn name(&self) -> &'static [u8] {
        match self {
            Self::Multistream => MULTISTREAM,
            Self::Noise => NOISE,
            Self::Yamux => YAMUX,
        }
    }
}

pub struct EncodedProtocol(Vec<u8>);

impl From<Protocol> for EncodedProtocol {
    fn from(protocol: Protocol) -> Self {
        let mut varint = [0; 10];
        let mut bytes = Vec::from(encode::usize(protocol.name().len(), &mut varint));
        bytes.extend_from_slice(protocol.name());

        Self(bytes)
    }
}

impl AsMut<[u8]> for EncodedProtocol {
    fn as_mut(&mut self) -> &mut [u8] {
        self.0.as_mut()
    }
}

impl AsRef<[u8]> for EncodedProtocol {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}
