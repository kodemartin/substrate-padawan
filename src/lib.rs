//! An elementary light-client library for substrate-based p2p networks.
//!
//! Built on top of [`libp2p`][]
use std::time::Duration;

use futures::StreamExt;
use libp2p::swarm::{dummy, NetworkBehaviour, Swarm, SwarmEvent};
use libp2p::{core, identity, multiaddr, noise, tcp, yamux, PeerId, Transport};
use tracing::info;

pub mod error;

/// The default timeout for incoming and outgoing connections
pub const DEFAULT_TIMEOUT_SECS: u64 = 20;

/// A swarm builder with minimal functionality.
pub struct SwarmPadawan<T: NetworkBehaviour> {
    keypair: identity::Keypair,
    peer_id: PeerId,
    timeout: Duration,
    behaviour: T,
}

type BoxedTransport = core::transport::Boxed<(PeerId, core::muxing::StreamMuxerBox)>;

impl<T: NetworkBehaviour> SwarmPadawan<T> {
    /// Create a new client with the given behaviour, and optional timeout parameter.
    pub fn new(behaviour: T, timeout: Option<Duration>) -> Self {
        let keypair = identity::Keypair::generate_ed25519();
        let peer_id = PeerId::from_public_key(&keypair.public());
        let timeout = timeout.unwrap_or_else(|| Duration::from_secs(DEFAULT_TIMEOUT_SECS));
        Self {
            keypair,
            peer_id,
            timeout,
            behaviour,
        }
    }

    /// The peer-id of the client
    pub fn peer_id(&self) -> &PeerId {
        &self.peer_id
    }

    /// Construct the [`Transport`][] associated with the client.
    ///
    /// It support all protocols required in the handshake between
    /// peers in substrate-based networks:
    ///
    /// * `multistream_select`
    /// * noise protocol
    /// * Yamux multiplexing
    fn transport(&self) -> BoxedTransport {
        tcp::tokio::Transport::new(tcp::Config::new().nodelay(true))
            .upgrade(core::upgrade::Version::V1)
            .authenticate(noise::NoiseAuthenticated::xx(&self.keypair).unwrap())
            .multiplex(yamux::YamuxConfig::default())
            .timeout(self.timeout)
            .boxed()
    }

    /// Build the swarm that will handle the tcp communication.
    pub fn swarm(self) -> Swarm<T> {
        Swarm::with_tokio_executor(self.transport(), self.behaviour, self.peer_id)
    }
}

/// Perform the basic handshake for substrate-based network peers
/// and close the connection upon success.
pub async fn handshake(
    padawan: SwarmPadawan<dummy::Behaviour>,
    remote: multiaddr::Multiaddr,
) -> error::Result<()> {
    let mut swarm = padawan.swarm();
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;
    swarm.dial(remote.clone())?;
    info!("Dialed remote: {}", remote);
    loop {
        match swarm.select_next_some().await {
            SwarmEvent::NewListenAddr { address, .. } => info!("Listening on {:?}", address),
            SwarmEvent::ConnectionEstablished {
                peer_id, endpoint, ..
            } => {
                info!("Established connection with {:?}", peer_id);
                if endpoint.get_remote_address() == &remote {
                    break;
                }
            }
            SwarmEvent::ConnectionClosed {
                peer_id, endpoint, ..
            } => {
                info!("Connection with {:?} is closed", peer_id);
                if endpoint.get_remote_address() == &remote {
                    break;
                }
            }
            _ => {}
        }
    }
    Ok(())
}
