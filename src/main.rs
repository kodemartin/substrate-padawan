use std::net::Ipv4Addr;

use clap::Parser;
use futures::StreamExt;
use libp2p::swarm::{dummy, Swarm, SwarmEvent};
use libp2p::{identity, multiaddr, PeerId};
use tracing::info;
use tracing_subscriber::FmtSubscriber;

fn use_tracing_subscriber() {
    let subscriber = FmtSubscriber::builder().finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
}

/// A command-line light-client that connects to a substrate
/// node using TCP.
///
/// Currently the client simply performs a handshake
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct CliArgs {
    /// The ip address of the peer node
    ip: String,
    /// The tcp port that the peer node listens to
    #[arg(long, short, default_value_t = 30333)]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use_tracing_subscriber();
    env_logger::init();

    let args = CliArgs::parse();
    let mut remote = multiaddr::Multiaddr::from(args.ip.parse::<Ipv4Addr>()?);
    remote.push(multiaddr::Protocol::Tcp(args.port));
    let keypair = identity::Keypair::generate_ed25519();
    let peer_id = PeerId::from(keypair.public());
    tracing::info!("Local peer id: {:?}", peer_id);

    let transport = libp2p::tokio_development_transport(keypair)?;
    let behaviour = dummy::Behaviour;

    let mut swarm = Swarm::with_tokio_executor(transport, behaviour, peer_id);
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
