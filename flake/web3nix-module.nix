# specific setups and tunes for hosting parity substrate p2p nodey
{ pkgs, config, ... }:
with pkgs.lib;
let
  cfg = config.web3nix;
in
{

  options = {
    web3nix.admin.email = pkgs.lib.mkOption {
      type = types.str;
    };
  };

  config = {
    security.acme.defaults.email = cfg.admin.email;
  };
}
