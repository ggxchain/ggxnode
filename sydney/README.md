## Docker

Due to the highly CPU dependent nature of 'cargo build' command, it's strongly recommended that you have at least 8 core enabled for this method.
It takes around 20 mins to complete with this suggested requirements, exponentially more if you use lesser proccessor power during the docker build operation.

From the repository's root directory execute following commands in order:
```bash
docker build -f ./sydney/Dockerfile -t golden-gate-node-sydney:local .
docker run -it --rm --name ggx-local-node -p 9944:9944 -p 9933:9933 -p 30333:30333 -v $(pwd):/tmp golden-gate-node-sydney:local /usr/src/app/target/release/golden-gate-node --ws-external --base-path=/data --chain /tmp/sydney/sydney.json --bootnodes /ip4/3.69.173.157/tcp/30333/p2p/12D3KooWHAuH2gKDCgoAVYciPgaoejVwXckEsjknr8AHHPEfdzgS --telemetry-url "ws://3.127.40.214:8001/submit 0"
```