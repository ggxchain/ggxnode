
# How to add new net

1. Buy domain and move it to AWS (other providers possible, so this is already encoded) and enable proper limits and billing
2. You have to enable AWS login locally mighty enough. Nix will eat what you have (like awscli2)
3. Tell `domain`, admin `email`, state encryption "age" key to nix via `email` and `domain` attributes in `flake.nix`
4. Set `UPTIME_TOKEN` into env.
5. run `terraform/base.nix` to apply base layer with node image
6. Generate `gen-node-key` for `.secret/node-a`, `-b`, etc
7. Run `terraform/testnet.nix` to deploy testnet and bind it to DNS
8. Get `ip` from `output` of 6, and put it into `flake.nix` for `deploy-testnet-node-a`, `deploy-testnet-node-b`, etc
9. Run relevant nix scripts  
