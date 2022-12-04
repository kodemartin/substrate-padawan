# `substrate-padawan`

An elementary library for implementing handshake-ready nodes of substrate-based networks.

The library exposes two implementations of the `libp2p` [connection ugrade][libp2p-conn-spec]

* A low-level implementation under `src/scratch` built on top of [snow][]
* A high-level implementation build on top of [libp2p][] in `src/swarm.rs` 

The connection-upgrade process involves the negotiation of the [noise][] and
[yamux][] protocols through the [multistream-select][mstream] protocol.

Upon successful negotiation of the [noise][] protocol the [noise-handshake][]
is implemented for both the dialer and listener role.

## Command-line applications

### Low-level (`substrate-scratch`)

```
$ cargo run --bin substrate-scratch -- --help

A command-line node implementing the libp2p-handshake

Usage: substrate-scratch [OPTIONS] <IP>

Arguments:
  <IP>
          The ip address of the peer node

Options:
  -p, --port <PORT>
          The tcp port that the peer node listens to

          [default: 30333]

  -l, --listen-port <LISTEN_PORT>
          The tcp port to listen for incoming connections.

          If not given the node listens to a random tcp port.

          [default: 0]

  -h, --help
          Print help information (use `-h` for a summary)

  -V, --version
          Print version information

```

### High-level (`substrate-swarm`)

```
$ cargo run --bin substrate-swarm -- --help

A command-line light-client that connects to a substrate node using TCP

Usage: substrate-padawan [OPTIONS] <IP>

Arguments:
  <IP>  The ip address of the peer node

Options:
  -p, --port <PORT>        The tcp port that the peer node listens to [default: 30333]
      --timeout <TIMEOUT>  The tcp timeout in secs
  -h, --help               Print help information
  -V, --version            Print version information

```

## Setup requirements

* [Rust](https://www.rust-lang.org/tools/install)

Minimum supported Rust version: `1.65`

## Testing

The application can be tested by running a local instance of [substrate-node-template][substrate-node] to simulate a network.

### `substrate-node`

Make sure to install all required system and rust components as instructed [here][node-install] and then run

```
$ cargo build --release
```

The node can be started locally with:

```
$ ./target/release/node-template \
     --chain local \
     --alice \
     --no-telemetry \
     -linfo,libp2p=debug
```

This will start a node at `/ip4/127.0.0.1/tcp/30333`.

### Connect to the substrate node and listen to new connections with  `substrate-scratch`

In another terminal run the `substrate-scratch` binary as follows:

```
$ RUST_LOG=debug cargo run --bin substrate-scratch -- 127.0.0.1 --port 30333 --listen-port 33333
Dec 04 13:20:27.857  INFO substrate_scratch: Listening on 127.0.0.1:33333
Dec 04 13:20:27.857  INFO substrate_padawan::scratch::connection: Local peer id: 12D3KooWPaffaDv9Wb8o5dj7fShVbAw1sVBmtDKVF8Eek1ixzAxQ
Dec 04 13:20:27.857  INFO substrate_padawan::scratch::connection: Initializing handshake
Dec 04 13:20:27.858  INFO substrate_padawan::scratch::connection: Negotiating protocol
Dec 04 13:20:27.860 DEBUG substrate_padawan::scratch::noise::libp2p: remote key Ed25519(PublicKey(compressed): c4646dfb308256a67676b18713cf9539cd61af2ec7b5c4d655181be2d99d595)
Dec 04 13:20:27.860  INFO substrate_padawan::scratch::noise::libp2p: Verified remote identity: PeerId("12D3KooWP2zu3EMZU8pJCDmyxNXEY2W7SzQSxLataA2DgCwYrh72")
Dec 04 13:20:27.861  INFO substrate_padawan::scratch::connection: Negotiating multiplex protocol
Dec 04 13:20:27.862  INFO substrate_padawan::scratch::connection: Connection established
```
It immediately performs the handshake with the `substrate-node`.

#### Incoming connections

To simulate incoming connections run a new node in a new terminal like so:

```
$ RUST_LOG=trace cargo run --bin substrate-scratch -- 127.0.0.1 --port 33333
Dec 04 13:24:22.082  INFO substrate_scratch: Listening on 127.0.0.1:33897
Dec 04 13:24:22.082  INFO substrate_padawan::scratch::connection: Local peer id: 12D3KooWGAfKTv2padYQ7eBGVPfH3wxNGLHC1LYUorNEc6yCnoe3
Dec 04 13:24:22.082  INFO substrate_padawan::scratch::connection: Initializing handshake
Dec 04 13:24:22.083  INFO substrate_padawan::scratch::connection: Negotiating protocol
Dec 04 13:24:22.085 DEBUG substrate_padawan::scratch::noise::libp2p: remote key Ed25519(PublicKey(compressed): cc814561fc1b83a24418b0f54aa5bb4a5fcaaaf12da157cac1339cb9fa4149)
Dec 04 13:24:22.086  INFO substrate_padawan::scratch::noise::libp2p: Verified remote identity: PeerId("12D3KooWPaffaDv9Wb8o5dj7fShVbAw1sVBmtDKVF8Eek1ixzAxQ")
Dec 04 13:24:22.086  INFO substrate_padawan::scratch::connection: Negotiating multiplex protocol
Dec 04 13:24:22.088  INFO substrate_padawan::scratch::connection: Connection established
```
### Connecting to a remote peer with the `substrate-swarm`

In another terminal run the `substrate-swarm` binary as follows:

```
$ cargo run --bin substrate-swarm -- 127.0.0.1 --port 30333
Nov 26 20:29:39.863  INFO substrate_padawan: Local peer id: PeerId("12D3KooWBY6VaEdkTCSXP3nXWBc6xGiSjM2MHbZPKK8ByDyJjaqp")
Nov 26 20:29:39.864  INFO substrate_padawan: Dialed remote: /ip4/127.0.0.1/tcp/30333
Nov 26 20:29:39.865  INFO substrate_padawan: Listening on "/ip4/127.0.0.1/tcp/38127"
Nov 26 20:29:39.868  INFO substrate_padawan: Established connection with PeerId("12D3KooWP2zu3EMZU8pJCDmyxNXEY2W7SzQSxLataA2DgCwYrh72")
```

## Additional resources

* [libp2p-specs][]
* [smoldot][smoldot]

[libp2p-specs]: https://github.com/libp2p/specs/
[libp2p-conn-spec]: https://github.com/libp2p/specs/blob/master/connections/README.md
[mstream]: https://github.com/multiformats/multistream-select
[node-install]: https://github.com/substrate-developer-hub/substrate-node-template#rust-setup
[noise]: http://noiseprotocol.org/
[yamux]: https://github.com/hashicorp/yamux/blob/master/spec.md
[libp2p]: https://github.com/libp2p/rust-libp2p
[smoldot]: https://github.com/paritytech/smoldot
[snow]: https://docs.rs/snow/latest/snow/index.html
[substrate-node]: https://github.com/substrate-developer-hub/substrate-node-template
[swarm]: https://docs.rs/libp2p/latest/libp2p/struct.Swarm.html
