# `substrate-padawan`

An elementary library for implementing a light-client to substrate-based networks.

Built on top of [libp2p][], the library exposes a [Swarm][swarm] builder that supports the basic protocols needed for a p2p handshake:

* [multistream-select][mstream]
* [noise][noise]
* [Yamux][yamux]

## Command-line application

There is also a binary available with the following API:

```
$ cargo run -- --help

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

### Connecting to a remote peer

In another terminal run `substrate-padawan` binary as follows:

```
$ cargo run -- 127.0.0.1 --port 30333
Nov 26 20:29:39.863  INFO substrate_padawan: Local peer id: PeerId("12D3KooWBY6VaEdkTCSXP3nXWBc6xGiSjM2MHbZPKK8ByDyJjaqp")
Nov 26 20:29:39.864  INFO substrate_padawan: Dialed remote: /ip4/127.0.0.1/tcp/30333
Nov 26 20:29:39.865  INFO substrate_padawan: Listening on "/ip4/127.0.0.1/tcp/38127"
Nov 26 20:29:39.868  INFO substrate_padawan: Established connection with PeerId("12D3KooWP2zu3EMZU8pJCDmyxNXEY2W7SzQSxLataA2DgCwYrh72")
```

## Additional resources

* [smoldot][smoldot]

[mstream]: https://github.com/multiformats/multistream-select
[node-install]: https://github.com/substrate-developer-hub/substrate-node-template#rust-setup
[noise]: http://noiseprotocol.org/
[yamux]: https://github.com/hashicorp/yamux/blob/master/spec.md
[libp2p]: https://github.com/libp2p/rust-libp2p
[smoldot]: https://github.com/paritytech/smoldot
[substrate-node]: https://github.com/substrate-developer-hub/substrate-node-template
[swarm]: https://docs.rs/libp2p/latest/libp2p/struct.Swarm.html
