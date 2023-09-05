# GGX Chain node set-up

## 1. Clone repo

To get started download this repository and navigate to `ggxnode` folder, e.g.:
```bash
git clone https://github.com/ggxchain/ggxnode.git
cd ggxchain
```

## 2. Install dependencies

The following dependencies are required to build the node:

#### Linux

```bash
# Install rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh 

# Install support software
sudo apt install build-essential protobuf-compiler libclang-dev

# Install wasm32-unknown-unknown target
rustup target add wasm32-unknown-unknown
rustup component add rust-src
```

#### MacOS

```bash
# Install rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh 

# Install protoc
brew install protobuf
```

#### Nix

```bash
# If you have nix package manager, downloads all necessary dependencies
nix develop --impure
```
## 3. Build & Run

### 3.1 With Docker

Due to the highly CPU dependent nature of 'cargo build' command, it's strongly recommended that you have at least 8 core enabled for this method.
It takes around 20 mins to complete with this suggested requirements, exponentially more if you use lesser processor power during the docker build operation.

From the repository's root directory execute following commands in order:

#### Sydney - our public testnet:
You have to create keys before running a validator node and store a backup of them.
**Make sure** to check [https://docs.ggxchain.io/](https://docs.ggxchain.io/) for requirements and documentation for running a Validator node.



```bash
docker build -f Dockerfile.sydney -t ggxchain-node:sydney .

mkdir -p data-sydney

docker run -d -it --restart=unless-stopped --ulimit nofile=100000:100000 \
    --name <INSERT_UNIQUE_NAME> \
    -p 127.0.0.1:9944:9944 \
    -p 127.0.0.1:9933:9933 \
    -p 127.0.0.1:9615:9615 \
    -p 0.0.0.0:30333:30333 \
    -v $(pwd)/custom-spec-files:/tmp \
    -v $(pwd)/data-sydney:/data-sydney \
    ggxchain-node:sydney \
    --wasm-execution Compiled \
    --database rocksdb \
    --rpc-cors all \
    --sync warp \
    --no-private-ip \
    --no-mdns \
    --state-pruning 256 \
    --blocks-pruning 256 \
    --node-key-type ed25519 \
    --node-key-file /data-sydney/node.key \
    --log info \
    --rpc-methods unsafe \
    --unsafe-rpc-external \
    --prometheus-external \
    --validator \
    --chain sydney \
    --base-path=/data-sydney \
    --bootnodes /dns/sun.sydney.ggxchain.io/tcp/30333/p2p/12D3KooWGmopnFNtQb2bo1irpjPLJUnmt9K4opTSHTMhYYobB8pC \
    --telemetry-url "wss://telemetry.sydney.ggxchain.io/submit 0"
```


#### Brooklyn - development network:

```bash
mkdir data-brooklyn

docker build -f Dockerfile.brooklyn -t ggxchain-node:brooklyn .

docker run \
    -it \
    --rm \
    --name ggx-local-node \
    -u $(id -g):$(id -u) \
    -p 30333:30333 \
    -v $(pwd)/custom-spec-files:/tmp \
    -v $(pwd)/data-brooklyn:/data-brooklyn \
    ggxchain-node:brooklyn \
    --base-path=/data-brooklyn \
    --chain brooklyn \
    --bootnodes /ip4/3.74.168.122/tcp/30333/p2p/12D3KooWCUvCEgrEqNHgMJjRmq2dYJmLX5jfcmMSte5SSwtsAsao \
    --telemetry-url "ws://test.telemetry.brooklyn.ggxchain.io/submit 0"
```


You can use the following optional flags:

| Flags                             | Description |
|-----------------------------------|---|
| `--validator`                     | Starts the node with the authority role and enables it to actively <br>participate in any consensus task that it can (for example, depending on<br> availability of local keys). |
| `--rpc-external`                  | Listens to all RPC interfaces. By default, the node only listens to <br>local RPC calls. If you set this command-line option, keep in mind that <br>that not all RPC methods are safe to be exposed publicly. Use an RPC <br>proxy server to filter out dangerous methods. For more information about<br> RPC methods that shouldn't be publicly exposed, see <br>Remote procedure calls. <br>Use `--unsafe-rpc-external` to suppress the warning if you understand the risks. |
| `--ws-external`                   | Listens to all Websocket interfaces. By default, the node only listens <br>locally. Keep in mind that not all RPC methods are safe to be exposed <br>publicly. You can use an RPC proxy server to filter out dangerous <br>methods. You can use `--unsafe-ws-external` to suppress the warning if you understand the risks. |
| `--unsafe-rpc-external`           | Listens to all RPC interfaces. This option is the same as <br>`--rpc-external`. |
| `--unsafe-ws-external`            | Listens to all Websocket interfaces. This option is the same as <br>`--ws-external` but doesn't warn you about it. |
| `--base-path <path>`              | Specifies a custom base path. |
| `--bootnodes <node-identifier>`   | Specifies a list of boot nodes identifiers for peer-to-peer communication. |
| `--chain <chain-specification>`   | Specifies the chain specification to use. You can set this option using a predefined chain specification name,<br>such as `dev`, `local`, or `staging`or you can specify the path to a file that contains the chain <br>specification, for example, the chain specification generated by using the build-spec subcommand. |
| `--name <name>`                   | Specifies the human-readable name for this node. The node name is reported to the telemetry server, if enabled. |
| `--password <password>`           | Specifies the password to use for the keystore. |
| `--telemetry-url <url verbosity>` | Specifies the URL of the telemetry server to connect to. You can pass <br>this flag multiple times to specify multiple telemetry endpoints. <br>Verbosity levels range from 0-9, with 0 denoting the least verbose. Use <br>the following format to specify the URL followed the verbosity option is `--telemetry-url 'wss://foo/bar 0'`. |

### 3.2 Without Docker

All required parameters (--name, -u, -p etc.) for `run` command you can take from Docker example.

#### Linux / MacOS

```bash
#Sydney:
cargo build --release  --features="sydney"
cargo run --release -p ggxchain-node --features "sydney"

#Brooklyn:
cargo build --release  --features="brooklyn"
cargo run --release -p ggxchain-node --features "brooklyn"
```
To run in dev mode add `-- --dev` flag to run command

#### nix

##### Sydney

```bash
nix build .#sydney-node
nix run .#sydney-node
```

To run in dev mode use

```bash
nix run .#single-fast sydney
```

To run 3-node network

```bash
nix run .#multi-fast sydney
```

To stop .#multi-fast or .#single-fast nodes

```bash
nix run .#prune-running
```

##### Brooklyn

```bash
nix build .#brooklyn-node
nix run .#brooklyn-node
```

To run in dev mode use

```bash
nix run .#single-fast`
```

To run 3-node network

```bash
nix run .#multi-fast
```

To stop .#multi-fast or .#single-fast nodes

```bash
nix run .#prune-running
```
