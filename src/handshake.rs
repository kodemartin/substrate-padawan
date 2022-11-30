use tokio::net::TcpStream;

use super::multistream_select::{mirror, EncodedProtocol, Protocol};

#[derive(Debug, Default, Clone, Copy, Hash)]
pub enum HandshakeState {
    #[default]
    Initialization,
    Negotiation,
    Noise,
    Failed,
}

impl HandshakeState {
    pub fn completed(&self) -> bool {
        matches!(self, Self::Noise)
    }

    pub fn failed(&self) -> bool {
        matches!(self, Self::Failed)
    }
}

#[derive(Debug)]
pub struct PadawanDialer {
    dialer: TcpStream,
    handshake: HandshakeState,
}

impl From<TcpStream> for PadawanDialer {
    fn from(stream: TcpStream) -> Self {
        Self {
            dialer: stream,
            handshake: Default::default(),
        }
    }
}

impl PadawanDialer {
    pub fn handshake_state(&self) -> HandshakeState {
        self.handshake
    }

    pub async fn handshake(&mut self) {
        while !self.handshake.completed() && !self.handshake.failed() {
            let (read, write) = self.dialer.split();
            match self.handshake {
                HandshakeState::Initialization => {
                    tracing::info!("Initializing handshake");
                    let hello = EncodedProtocol::from(Protocol::Multistream);
                    if mirror(read, write, hello.as_bytes()).await {
                        self.handshake = HandshakeState::Negotiation;
                    } else {
                        self.handshake = HandshakeState::Failed;
                    }
                }
                HandshakeState::Negotiation => {
                    tracing::info!("Negotiating protocol");
                    let noise = EncodedProtocol::from(Protocol::Noise);
                    if mirror(read, write, noise.as_bytes()).await {
                        self.handshake = HandshakeState::Noise;
                    } else {
                        self.handshake = HandshakeState::Failed;
                    }
                }
                HandshakeState::Noise => todo!(),
                HandshakeState::Failed => unreachable!(),
            }
        }
    }
}
