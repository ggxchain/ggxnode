# Note: This is currently designed to simplify development
# To get a smaller docker image, there should be 2 images generated, in 2 stages.
FROM rustlang/rust:nightly as builder

## Upcd dates core parts
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

# Install rust wasm. Needed for substrate wasm engine
RUN rustup target add wasm32-unknown-unknown

WORKDIR /app
COPY  . .

# Download Frontier repo
RUN cargo build --release --config net.git-fetch-with-cli=true

FROM debian:sid

# Copy the node binary.
COPY --from=builder /app/target/release/golden-gate-node /app/target/release/golden-gate-node
COPY --from=builder /app/chainspecs/brooklyn/customSpecRaw.json /
