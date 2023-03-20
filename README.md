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

#### Docker example

You can use the Docker image with all the dependencies installed.

```bash
docker build -t golden-gate-env -f containers/enviroment . # build the docker image
docker run -it -p 9944:9944 -p 9933:9933 -v $(pwd):/golden-gate golden-gate-env bash # run the docker image
cd golden-gate # go to the root of the repository
cargo run --release -- --dev --ws-external --rpc-external # start the node
```

#### Build

```bash
cargo build --release
```

#### Run

```bash
cargo run --release -- --dev
```
