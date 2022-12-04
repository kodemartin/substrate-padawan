//! Handle connections between peers implementing the `libp2p` networking stack.
use futures::{stream::FuturesUnordered, StreamExt};
use libp2p::{identity, PeerId};
use tokio::net::{TcpListener, TcpStream};

use crate::error::PadawanError;

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

/// The local node
///
/// Capable of bidirectional communication with its peers.
pub struct Padawan {
    dialer: Connection,
    listener: TcpListener,
    keypair: identity::Keypair,
    peer_id: PeerId,
}

impl Padawan {
    /// Create a new local node
    pub fn new(dialer: TcpStream, listener: TcpListener) -> Self {
        let keypair = identity::Keypair::generate_ed25519();
        let peer_id = PeerId::from_public_key(&keypair.public());
        tracing::info!("Local peer id: {}", peer_id);
        Self {
            dialer: Connection::new(dialer, keypair.clone(), Some(peer_id)),
            listener,
            keypair,
            peer_id,
        }
    }

    /// Start dialing and accepting new connections
    pub async fn start(mut self) -> Result<(), PadawanError> {
        let mut dial_listen = FuturesUnordered::new();
        dial_listen.push(tokio::spawn(async move { self.dialer.dial().await }));
        dial_listen.push(tokio::spawn(async move {
            loop {
                let (keypair, peer_id) = (self.keypair.clone(), self.peer_id);
                if let Ok((socket, addr)) = self.listener.accept().await {
                    tracing::info!("Incoming connection {}", addr);
                    tokio::spawn(async move {
                        let mut listener = Connection::new(socket, keypair, Some(peer_id));
                        listener.listen().await
                    });
                }
            }
        }));
        while dial_listen.next().await.is_some() {
            // We keep awaiting for both the dialer and the listener
        }
        Ok(())
    }
}

/// Represent a connection of the local node acting either as a dialer or listener
pub struct Connection {
    wire: TcpStream,
    state: HandshakeState,
    keypair: identity::Keypair,
    peer_id: PeerId,
}

impl From<TcpStream> for Connection {
    /// Create a new connection with auto-generated [`PeerId`][].
    fn from(wire: TcpStream) -> Self {
        let keypair = identity::Keypair::generate_ed25519();
        let peer_id = PeerId::from_public_key(&keypair.public());
        Self {
            wire,
            state: Default::default(),
            keypair,
            peer_id,
        }
    }
}

impl Connection {
    /// Create a new connection associated with the given [`Keypair`][`identity::Keypair`].
    pub fn new(wire: TcpStream, keypair: identity::Keypair, peer_id: Option<PeerId>) -> Self {
        let peer_id = peer_id.unwrap_or_else(|| PeerId::from_public_key(&keypair.public()));
        Self {
            wire,
            state: Default::default(),
            keypair,
            peer_id,
        }
    }

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

    /// Perform the handshake with the remote peer as a listener
    pub async fn listen(&mut self) -> Result<(), PadawanError> {
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
                    if mirror::listen(&mut read, &mut write, noise).await.is_ok() {
                        self.state = HandshakeState::Noise;
                    } else {
                        self.state = HandshakeState::Failed;
                    }
                }
                HandshakeState::Noise => {
                    let mut handshake = noise::libp2p::NoiseHandshake::listener()?;
                    handshake.recv_hello(&mut read).await?;
                    handshake.send_identity(&mut write, &self.keypair).await?;
                    handshake.recv_identity(&mut read).await?;
                    let transport = handshake.into_inner().try_into()?;
                    self.state = HandshakeState::Multiplex(Box::new(transport));
                }
                HandshakeState::Multiplex(ref mut transport) => {
                    tracing::info!("Negotiating multiplex protocol");
                    let headers = Protocol::Multistream;
                    if mirror::listen_noise(&mut read, &mut write, headers, transport)
                        .await
                        .is_err()
                    {
                        self.state = HandshakeState::Failed;
                        continue;
                    }
                    let yamux = Protocol::Yamux;
                    if mirror::listen_noise(&mut read, &mut write, yamux, transport)
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
