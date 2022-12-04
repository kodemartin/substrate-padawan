//! Logic and types for the `libp2p` implementation
//! of the noise protocol.
#![allow(clippy::derive_partial_eq_without_eq)]

use libp2p::identity;
use prost::Message;
use tokio::net::tcp;

use super::wire;
use super::Handshake;
use crate::error::PadawanError;

// The protobuf types representing the handshake payload
//
// See `/src/noise/proto/handshake_payload.proto`
include!(concat!(env!("OUT_DIR"), "/payload.rs"));

impl NoiseHandshakePayload {
    pub fn verify_identity(&self, identity: Identity) -> Result<(), PadawanError> {
        let remote_key = identity::PublicKey::from_protobuf_encoding(self.identity_key())?;
        tracing::debug!("remote key {:?}", remote_key);

        let sig = self
            .identity_sig
            .as_ref()
            .ok_or(PadawanError::IdVerification)?;
        if !remote_key.verify(&identity.into_message(), sig) {
            return Err(PadawanError::IdVerification);
        }
        tracing::info!(
            "Verified remote identity: {:?}",
            libp2p::PeerId::from_public_key(&remote_key)
        );
        Ok(())
    }
}

/// The peer identity as represented in the noise handshake
pub struct Identity<'a>(&'a [u8]);

impl<'a> Identity<'a> {
    const PREFIX: &[u8] = b"noise-libp2p-static-key:";

    /// Create a new identity from the raw key bytes
    pub fn new(key: &'a [u8]) -> Self {
        Self(key)
    }

    /// Transfrom the identity into a message for signing
    pub fn into_message(self) -> Vec<u8> {
        let mut msg = Vec::from(Self::PREFIX);
        msg.extend_from_slice(self.0);
        msg
    }
}

/// Represents the `libp2p` implementation of the noise handshake
pub struct NoiseHandshake(Handshake);

impl From<Handshake> for NoiseHandshake {
    fn from(protocol_state: Handshake) -> Self {
        Self(protocol_state)
    }
}

impl NoiseHandshake {
    /// Build an initiator handshake state
    pub fn dialer() -> Result<Self, PadawanError> {
        Ok(Self::from(Handshake::build_initiator()?))
    }

    /// Build a responder handshake state
    pub fn listener() -> Result<Self, PadawanError> {
        Ok(Self::from(Handshake::build_responder()?))
    }

    /// Get the inner stateful buffer of the noise-handshake state
    pub fn into_inner(self) -> Handshake {
        self.0
    }

    /// Dialer initial communication
    pub async fn hello<'a>(&mut self, write: &mut tcp::WriteHalf<'a>) -> Result<(), PadawanError> {
        let encrypted = self.0.encrypt()?;
        let n = wire::send(write, encrypted).await?;
        tracing::trace!("Noise hello: {:?} bytes", n);
        Ok(())
    }

    /// Listener initial communication
    pub async fn recv_hello<'a>(
        &mut self,
        read: &mut tcp::ReadHalf<'a>,
    ) -> Result<(), PadawanError> {
        wire::recv(read, self.0.buffer().encrypted()).await?;
        self.0.decrypt()?;
        Ok(())
    }

    /// Receive and verify identity from the remote peer
    pub async fn recv_identity<'a>(
        &mut self,
        read: &mut tcp::ReadHalf<'a>,
    ) -> Result<(), PadawanError> {
        wire::recv(read, self.0.buffer().encrypted()).await?;
        let decrypted = self.0.decrypt()?;
        let payload = NoiseHandshakePayload::decode(decrypted.as_slice())?;
        let remote_key = self
            .0
            .remote_static()
            .ok_or(PadawanError::MissingRemoteNoiseKey)?;
        payload.verify_identity(Identity::new(remote_key))?;
        Ok(())
    }

    /// Construct and send identity payload for a local peer
    pub async fn send_identity<'a>(
        &mut self,
        write: &mut tcp::WriteHalf<'a>,
        keypair: &identity::Keypair,
    ) -> Result<(), PadawanError> {
        let msg = Identity::new(self.0.local_static()).into_message();
        let payload = NoiseHandshakePayload {
            identity_key: Some(keypair.public().to_protobuf_encoding()),
            identity_sig: Some(keypair.sign(&msg)?),
            extensions: None,
        };
        payload.encode(self.0.buffer().write())?;
        let encrypted = self.0.encrypt()?;
        let n = wire::send(write, encrypted).await?;
        tracing::trace!("Sent noise identity: {:?} bytes", n);
        Ok(())
    }
}
