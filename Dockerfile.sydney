FROM rustlang/rust:nightly as builder

RUN apt-get update -y && \
    apt-get install -y libclang-dev && \
    apt-get install -y libudev-dev && \
    apt-get install -y libssl-dev && \
    apt-get install -y pkg-config && \
    apt-get install -y gcc && \
    apt-get install -y cmake && \
    apt-get install -y git && \
    apt-get install -y gcc && \
    apt-get install -y protobuf-compiler

RUN rustup target add wasm32-unknown-unknown

WORKDIR /usr/src/app

COPY . .

RUN cargo build --release --package golden-gate-node --no-default-features "--features=aura,with-rocksdb-weights,mainnet" --config net.git-fetch-with-cli=true


FROM debian:11 as production

ENV HOME /usr/src/app
WORKDIR $HOME

COPY --from=builder $HOME/target/release/golden-gate-node ./target/release/golden-gate-node
