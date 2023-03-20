# How to add more validators

0. [You are running net](net.md)
1. `gen-node-key` for `.secret/node-d`
2. Add `node-d` into `terraform/testnet.nix` in places where `node-b` mentioned (instances, zones, keys). Run.
3. Copy `node-d` IP.
4. In `flake.nix` add `node-d` in all places where mentioned `node-b`. Run `deploy-node-d` routine.
5. Follow [guide](../examples/adding-new-validator/README.md) on how to get validator public keys set and add it into chain.
