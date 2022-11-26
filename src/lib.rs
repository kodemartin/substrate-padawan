use libp2p::ping;
use libp2p::swarm::{keep_alive, NetworkBehaviour};
use void::Void;

#[derive(NetworkBehaviour, Default)]
#[behaviour(out_event = "Event")]
pub struct PingAliveBehaviour {
    ping: ping::Behaviour,
    keep_alive: keep_alive::Behaviour,
}

impl PingAliveBehaviour {
    pub fn new(ping: ping::Behaviour, keep_alive: keep_alive::Behaviour) -> Self {
        Self { ping, keep_alive }
    }
}

#[derive(Debug)]
pub enum Event {
    KeepAlive(Void),
    Ping(ping::Event),
}

impl From<Void> for Event {
    fn from(event: Void) -> Self {
        Self::KeepAlive(event)
    }
}

impl From<ping::Event> for Event {
    fn from(event: ping::Event) -> Self {
        Self::Ping(event)
    }
}
