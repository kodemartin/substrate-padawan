use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{ReadHalf, WriteHalf};

use unsigned_varint::encode;

const MULTISTREAM: &[u8] = b"/multistream/1.0.0\n";
const NOISE: &[u8] = b"/noise\n";

#[derive(Debug, Clone, Copy, Hash)]
pub enum Protocol {
    Multistream,
    Noise,
}

impl Protocol {
    pub fn name(&self) -> &'static [u8] {
        match self {
            Self::Multistream => MULTISTREAM,
            Self::Noise => NOISE,
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

impl EncodedProtocol {
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_ref()
    }
}

pub async fn mirror<'a>(mut read: ReadHalf<'a>, mut write: WriteHalf<'a>, bytes: &[u8]) -> bool {
    let (mut send, mut recv) = (false, false);
    let mut response = vec![0_u8; bytes.len()];
    loop {
        tokio::select!(
            Ok(n_read) = read.read_exact(&mut response), if !recv  => {
                tracing::debug!("read {:?} bytes", response.as_slice());
                recv = n_read > 0;
            },
            Ok(_) = write.write_all(bytes), if !send => {
                tracing::debug!("wrote {:?} ", bytes);
                send = true;
            },
            else => break
        );
    }
    response == bytes
}
