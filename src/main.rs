use std::net::Ipv4Addr;
use std::time::Duration;

use clap::Parser;
use libp2p::multiaddr;
use libp2p::swarm::dummy;
use substrate_padawan::{error, handshake, SwarmPadawan};
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
    /// The tcp timeout in secs
    #[arg(long)]
    timeout: Option<u64>,
}

#[tokio::main]
async fn main() -> error::Result<()> {
    use_tracing_subscriber();
    env_logger::init();

    let args = CliArgs::parse();
    let mut remote = multiaddr::Multiaddr::from(args.ip.parse::<Ipv4Addr>()?);
    remote.push(multiaddr::Protocol::Tcp(args.port));

    let padawan = SwarmPadawan::new(dummy::Behaviour, args.timeout.map(Duration::from_secs));
    tracing::info!("Local peer id: {:?}", padawan.peer_id());

    handshake(padawan, remote).await
}
