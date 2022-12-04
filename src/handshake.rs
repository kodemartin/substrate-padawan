use libp2p::{identity, PeerId};
use tokio::net::TcpStream;

use super::error::PadawanError;
use super::multistream_select::{mirror, Protocol};
use super::noise;

/// Represent the state of the `libp2p` upgrade negotionation
/// that includes:
///
/// * `multistream_select`
/// * `noise` handshake
/// * `yamux` negotiation
#[derive(Default)]
pub enum HandshakeState {
    Established,
    #[default]
    Initialization,
    Negotiation,
    Noise,
    Multiplex(Box<noise::Transport>),
    Failed,
}

impl HandshakeState {
    pub fn completed(&self) -> bool {
        matches!(self, Self::Established)
    }
}

/// Represent the local node
pub struct Padawan {
    wire: TcpStream,
    state: HandshakeState,
    keypair: identity::Keypair,
    peer_id: PeerId,
}

impl From<TcpStream> for Padawan {
    fn from(stream: TcpStream) -> Self {
        let keypair = identity::Keypair::generate_ed25519();
        let peer_id = PeerId::from_public_key(&keypair.public());
        Self {
            wire: stream,
            state: Default::default(),
            keypair,
            peer_id,
        }
    }
}

impl Padawan {
    pub fn peer_id(&self) -> &PeerId {
        &self.peer_id
    }

    /// The inner state of the handshake
    pub fn handshake_state(&self) -> &HandshakeState {
        &self.state
    }

    /// Perform the handshake with the remote peer as a dialer
    pub async fn dial(&mut self) -> Result<(), PadawanError> {
        let (mut read, mut write) = self.wire.split();
        loop {
            match self.state {
                HandshakeState::Initialization => {
                    tracing::info!("Initializing handshake");
                    let hello = Protocol::Multistream;
                    if let Ok(true) = mirror::concurrent(&mut read, &mut write, hello).await {
                        self.state = HandshakeState::Negotiation;
                    } else {
                        self.state = HandshakeState::Failed;
                    }
                }
                HandshakeState::Negotiation => {
                    tracing::info!("Negotiating protocol");
                    let noise = Protocol::Noise;
                    if let Ok(true) = mirror::dial(&mut read, &mut write, noise).await {
                        self.state = HandshakeState::Noise;
                    } else {
                        self.state = HandshakeState::Failed;
                    }
                }
                HandshakeState::Noise => {
                    let mut handshake = noise::libp2p::NoiseHandshake::dialer()?;
                    handshake.hello(&mut write).await?;
                    handshake.recv_identity(&mut read).await?;
                    handshake.send_identity(&mut write, &self.keypair).await?;
                    let transport = handshake.into_inner().try_into()?;
                    self.state = HandshakeState::Multiplex(Box::new(transport));
                }
                HandshakeState::Multiplex(ref mut transport) => {
                    tracing::info!("Negotiating multiplex protocol");
                    let headers = Protocol::Multistream;
                    if mirror::dial_noise(&mut read, &mut write, headers, transport)
                        .await
                        .is_err()
                    {
                        self.state = HandshakeState::Failed;
                        continue;
                    }
                    let yamux = Protocol::Yamux;
                    if mirror::dial_noise(&mut read, &mut write, yamux, transport)
                        .await
                        .is_ok()
                    {
                        self.state = HandshakeState::Established;
                        tracing::info!("Connection established");
                    } else {
                        self.state = HandshakeState::Failed;
                    }
                }
                HandshakeState::Failed => return Err(PadawanError::HandshakeFailed),
                HandshakeState::Established => break,
            }
        }
        Ok(())
    }
}
