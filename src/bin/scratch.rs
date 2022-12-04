use std::net::Ipv4Addr;

use clap::Parser;
use substrate_padawan::{error, scratch::connection};
use tokio::net::{TcpListener, TcpStream};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

fn use_tracing_subscriber() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .with_env_filter(EnvFilter::from_default_env())
        .finish();

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
    let ipv4 = args.ip.parse::<Ipv4Addr>()?;
    let dialer = TcpStream::connect((ipv4, args.port)).await?;
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    tracing::info!("Listening on {}", listener.local_addr()?);
    connection::Padawan::new(dialer, listener).start().await
}
