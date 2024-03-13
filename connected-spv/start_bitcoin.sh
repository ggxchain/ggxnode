#!/bin/bash -xe

cat <<EOF > /root/.bitcoin/bitcoin.conf
regtest=1
txindex=1
server=1

[regtest]
port=18444
rpcport=18443
rpcbind=127.0.0.1
EOF


bitcoind

### Prepare Bitcoin:
# $ bitcoin-cli createwallet "bohdan"
# $ bitcoin-cli getnewaddress
# bcrt1qunsup9cs59flsvpj6j9sa4hzg9atazxl9lgpeu
# $ bitcoin-cli generatetoaddress 100 bcrt1qunsup9cs59flsvpj6j9sa4hzg9atazxl9lgpeu


### Prepare SPV:
# $ cd connected-spv
# $ cargo run -- --connect 127.0.0.1:18444 --regtest
