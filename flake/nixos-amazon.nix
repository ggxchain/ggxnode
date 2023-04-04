({ pkgs, lib, config, options, specialArgs, modulesPath }:
let
  ws_port = 9944;
  allowedTCPPorts = [ 80 443 8080 8443 5000 5001 3000 9988 ws_port 9933 30333 9615 ];
in
{
  system.stateVersion = "22.11";
  nix = {
    package = pkgs.nixFlakes;
    extraOptions = ''
      experimental-features = nix-command flakes
      sandbox = relaxed
    '';
    settings = {
      trusted-users = [ "root" "admin" ];
      extra-substituters = [ "https://cache.nixos.org" "https://golden-gate-ggx.cachix.org" ];
      extra-trusted-public-keys = [ "cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY=" "golden-gate-ggx.cachix.org-1:h2zGCI9FqxUS7HxnZJDHaJzbN4iTsWvBcETdd+/0ZD4=" ];
    };
  };

  imports =
    [ "${toString modulesPath}/virtualisation/amazon-image.nix" ];
  services.openssh.passwordAuthentication = lib.mkForce true;
  services.openssh.permitRootLogin = lib.mkForce "yes";

  security = {
    acme = {
      acceptTerms = true;
    };
  };
  services.nginx.enable = true;
  services.nginx.virtualHosts = {
    "_" = {
      # addSSL = true;
      #enableACME = true;
      root = "/var/www/default";
      # just stub for root page, can route to any usefull info or landing
      locations."/" = {
        root = pkgs.runCommand "testdir" { } ''
          mkdir "$out"
          echo "golden gate base image" > "$out/index.html"
        '';
      };
      locations."/substrate/client" = {
        # any all to external servers is routed to node
        proxyPass = "http://127.0.0.1:${builtins.toString ws_port}";
        proxyWebsockets = true;
      };
    };
  };
  services.nginx.logError = "stderr debug";

  networking.firewall = {
    enable = true;
    inherit allowedTCPPorts;
    # not secure
    allowedTCPPortRanges = [{
      from = 80;
      to = 40000;
    }];
  };

  environment.systemPackages = with pkgs; [ git helix curl websocat jq]; # add here various substrate tools, like subkey, subxt
})
