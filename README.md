# 🛠🚧🏗 Under Construction 🛠🚧🏗

## Node set-up

### Dependencies

The following dependencies are required to run the project:

* rust, wasm32-unknown-unknown target
* protobuf
* dylint

#### Ubuntu example

```bash
# Install rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh 

# Install wasm32-unknown-unknown target
rustup target add wasm32-unknown-unknown
rustup component add rust-src

# Install protobuf
sudo apt install protobuf-compiler

# Install dylint
cargo install cargo-dylint dylint-link
```

#### Nix example

```bash
# Downloads all necessary dependendencies
nix develop --impure
```

## Docker

Due to the highly CPU dependent nature of 'cargo build' command, it's strongly recommended that you have at least 8 core enabled for this method.
It takes around 20 mins to complete with this suggested requirements, exponentially more if you use lesser proccessor power during the docker build operation.

From the repository's root directory execute following commands in order:

Brooklyn:
```bash
docker build -f Dockerfile.brooklyn -t golden-gate-node:brooklyn .
docker run -it --rm --name ggx-local-node -p 9944:9944 -p 9933:9933 -p 30333:30333 -v $(pwd)/custom-spec-files:/tmp golden-gate-node:brooklyn /usr/src/app/target/release/golden-gate-node --dev --ws-external --base-path=/data-brooklyn --chain /tmp/brooklyn.json
```
Sydney:
```bash
docker build -f Dockerfile.sydney -t golden-gate-node:sydney .
docker run -it --rm --name ggx-local-node -p 9944:9944 -p 9933:9933 -p 30333:30333 -v $(pwd)/custom-spec-files:/tmp golden-gate-node:sydney /usr/src/app/target/release/golden-gate-node --ws-external --base-path=/data-sydney --chain /tmp/sydney.json --bootnodes /ip4/3.69.173.157/tcp/30333/p2p/12D3KooWHAuH2gKDCgoAVYciPgaoejVwXckEsjknr8AHHPEfdzgS --telemetry-url "ws://3.127.40.214:8001/submit 0"
```

#### Build

```bash
cargo build --release
# or using nix
nix build .#node
```

#### Run

```bash
cargo run --release -- --dev
# or using nix
nix run .#single-fast # to run an one node network
nix run .#multi-fast # to run 3-node network
nix run .#prune-running # to stop nodes
```
