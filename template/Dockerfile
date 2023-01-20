# Note: This is currently designed to simplify development
# To get a smaller docker image, there should be 2 images generated, in 2 stages.
FROM rustlang/rust:nightly as builder

# Upcd dates core parts
RUN apt-get update -y && \
	apt-get install -y cmake pkg-config libssl-dev git gcc build-essential clang libclang-dev protobuf-compiler

# Install rust wasm. Needed for substrate wasm engine
RUN rustup target add wasm32-unknown-unknown

WORKDIR /frontier
COPY  . .

# Download Frontier repo
RUN rm rust-toolchain
RUN cargo build --release

FROM debian:sid

# Copy the node binary.
COPY --from=builder /frontier/target/release/frontier-template-node /frontier/target/release/frontier-template-node
