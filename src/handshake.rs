use libp2p::{identity, PeerId};
use tokio::net::TcpStream;

use super::error::PadawanError;
use super::multistream_select::{mirror, EncodedProtocol, Protocol};
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

    pub fn failed(&self) -> bool {
        matches!(self, Self::Failed)
    }
}

/// Represent the local node as dialer
pub struct PadawanDialer {
    dialer: TcpStream,
    state: HandshakeState,
    keypair: identity::Keypair,
    peer_id: PeerId,
}

impl From<TcpStream> for PadawanDialer {
    fn from(stream: TcpStream) -> Self {
        let keypair = identity::Keypair::generate_ed25519();
        let peer_id = PeerId::from_public_key(&keypair.public());
        Self {
            dialer: stream,
            state: Default::default(),
            keypair,
            peer_id,
        }
    }
}

impl PadawanDialer {
    pub fn peer_id(&self) -> &PeerId {
        &self.peer_id
    }

    /// The inner state of the handshake
    pub fn handshake_state(&self) -> &HandshakeState {
        &self.state
    }

    /// Perform the handshake with the remote peer
    pub async fn handshake(&mut self) -> Result<(), PadawanError> {
        let (mut read, mut write) = self.dialer.split();
        while !self.state.completed() && !self.state.failed() {
            match self.state {
                HandshakeState::Initialization => {
                    tracing::info!("Initializing handshake");
                    let hello = EncodedProtocol::from(Protocol::Multistream);
                    if mirror::concurrent(&mut read, &mut write, hello.as_bytes()).await {
                        self.state = HandshakeState::Negotiation;
                    } else {
                        self.state = HandshakeState::Failed;
                    }
                }
                HandshakeState::Negotiation => {
                    tracing::info!("Negotiating protocol");
                    let noise = EncodedProtocol::from(Protocol::Noise);
                    if mirror::dial(&mut read, &mut write, noise.as_bytes()).await {
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
                    let headers = EncodedProtocol::from(Protocol::Multistream);
                    mirror::dial_noise(&mut read, &mut write, headers, transport).await?;
                    let yamux = EncodedProtocol::from(Protocol::Yamux);
                    mirror::dial_noise(&mut read, &mut write, yamux, transport).await?;
                    self.state = HandshakeState::Established;
                    tracing::info!("Connection established");
                }
                HandshakeState::Failed => unreachable!(),
                _ => {}
            }
        }
        Ok(())
    }
}