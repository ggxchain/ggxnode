# eth-light-client

## Install dependencies

```bash
sudo apt install libsqlite3-dev
```

## Run

```bash
SMART_CONTRACT_ADDRESS="0xdAC17F958D2ee523a2206206994597C13D831ec7" \
CONSENSUS_RPC="https://www.lightclientdata.org" \
UNTRUSTED_RPC="https://eth-mainnet.g.alchemy.com/v2/YOUR_TOKEN" \
DB_PATH="/tmp/eth-light-client.sqlite" \
HELIOS_HOME_PATH="/tmp/helios" \
SERVER_PORT=5800 \
MAX_SUPPORTED_LOGS_NUMBER=50 \
RUST_LOG=debug \
cargo run
```
