# 🛠🚧🏗 Under Construction 🛠🚧🏗 

## Node set-up

### Dependencies
The following dependencies are required to run the project:
* rust, wasm32-unknown-unknown target
* protobuf
* dylint

#### Ubuntu example
```
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

#### Build
```
cargo build --release
```

#### Run
```
cargo run --release -- --dev
```