//! Implementation of the `noise-libp2p` protocol as specified
//! in https://github.com/libp2p/specs/tree/master/noise
use snow::{Builder, HandshakeState, TransportState};

use crate::error::PadawanError;

pub mod libp2p;
pub mod wire;

/// Reserved space for the size increase caused by encryption
const ENCRYPTION_INFLATION_SIZE: usize = 1024;

/// The supported noise handshake pattern
static PATTERN: &str = "Noise_XX_25519_ChaChaPoly_SHA256";

/// Common behaviour exposed by [`snow`][] state abstractions.
pub trait NoiseState {
    /// The remote peer static public key
    fn remote_static(&self) -> Option<&[u8]>;

    /// Decrypt a message into the given `plaintext` buffer.
    ///
    /// Returns the number of bytes encrypted.
    fn decrypt(&mut self, encrypted: &[u8], plaintext: &mut [u8]) -> Result<usize, PadawanError>;

    /// Decrypt a message into the given `plaintext` buffer.
    ///
    /// Returns the number of bytes encrypted.
    fn encrypt(&mut self, plaintext: &[u8], encrypted: &mut [u8]) -> Result<usize, PadawanError>;
}

impl NoiseState for HandshakeState {
    fn remote_static(&self) -> Option<&[u8]> {
        self.get_remote_static()
    }

    fn decrypt(&mut self, encrypted: &[u8], plaintext: &mut [u8]) -> Result<usize, PadawanError> {
        Ok(self.read_message(encrypted, plaintext)?)
    }

    fn encrypt(&mut self, plaintext: &[u8], encrypted: &mut [u8]) -> Result<usize, PadawanError> {
        Ok(self.write_message(plaintext, encrypted)?)
    }
}

impl NoiseState for TransportState {
    fn remote_static(&self) -> Option<&[u8]> {
        self.get_remote_static()
    }

    fn decrypt(&mut self, encrypted: &[u8], plaintext: &mut [u8]) -> Result<usize, PadawanError> {
        Ok(self.read_message(encrypted, plaintext)?)
    }

    fn encrypt(&mut self, plaintext: &[u8], encrypted: &mut [u8]) -> Result<usize, PadawanError> {
        Ok(self.write_message(plaintext, encrypted)?)
    }
}

/// Encapsulate buffers for read, write, and encrypted data.
#[derive(Debug, Default, Clone)]
pub struct Buffer {
    read: Vec<u8>,
    write: Vec<u8>,
    encrypted: Vec<u8>,
}

impl Buffer {
    pub fn read(&mut self) -> &mut Vec<u8> {
        &mut self.read
    }

    pub fn write(&mut self) -> &mut Vec<u8> {
        &mut self.write
    }

    pub fn encrypted(&mut self) -> &mut Vec<u8> {
        &mut self.encrypted
    }
}

/// Handle encryption/decryption of data based on the state
/// of the noise protocol
pub struct StatefulBuf<T: NoiseState> {
    state: T,
    buffer: Buffer,
    keypair: snow::Keypair,
}

impl<T: NoiseState> StatefulBuf<T> {
    /// Get the remote static noise key
    pub fn remote_static(&self) -> Option<&[u8]> {
        self.state.remote_static()
    }

    /// Get the local static noise key
    pub fn local_static(&self) -> &[u8] {
        &self.keypair.public
    }

    /// Decrypt encrypted data from the internal buffer.
    ///
    /// Returns a mutable reference to the decrypted data.
    pub fn decrypt(&mut self) -> Result<&mut Vec<u8>, PadawanError> {
        self.buffer.read.resize(self.buffer.encrypted.len(), 0);
        let n = self
            .state
            .decrypt(&self.buffer.encrypted, &mut self.buffer.read)?;
        self.buffer.read.truncate(n);
        Ok(self.buffer.read())
    }

    /// Encrypt the data of the internal write buffer.
    ///
    /// Returns a mutable reference to the encrypted data.
    pub fn encrypt(&mut self) -> Result<&mut Vec<u8>, PadawanError> {
        self.buffer
            .encrypted
            .resize(self.buffer.write.len() + ENCRYPTION_INFLATION_SIZE, 0);
        let n = self
            .state
            .encrypt(&self.buffer.write, &mut self.buffer.encrypted)?;
        self.buffer.encrypted.truncate(n);
        Ok(self.buffer.encrypted())
    }

    /// Get the internal buffer
    pub fn buffer(&mut self) -> &mut Buffer {
        &mut self.buffer
    }
}

/// The stateful buffer to be used in the handshake phase
pub type Handshake = StatefulBuf<HandshakeState>;

impl Handshake {
    /// Build a handshake for the side that will send the first message
    pub fn build_initiator() -> Result<Self, PadawanError> {
        let builder = Builder::new(PATTERN.parse()?);
        let keypair = builder.generate_keypair()?;
        let state = builder
            .local_private_key(&keypair.private)
            .build_initiator()?;
        Ok(Self {
            state,
            buffer: Default::default(),
            keypair,
        })
    }
}

/// The stateful buffer to be used in the transport phase
pub type Transport = StatefulBuf<TransportState>;

impl TryFrom<Handshake> for Transport {
    type Error = PadawanError;

    fn try_from(handshake: Handshake) -> Result<Self, Self::Error> {
        Ok(Self {
            state: handshake.state.try_into()?,
            buffer: handshake.buffer,
            keypair: handshake.keypair,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_initiator() {
        assert!(Handshake::build_initiator().is_ok());
    }
}
