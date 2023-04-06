{
  # like Cargo.toml or package.json dependencies, but on meta level (tools to run mentined files)
  inputs = {
    # base packages
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-22.11";

    # shells
    devenv.url = "github:cachix/devenv";

    # rust version
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };

    # mac/linux arm/x86 support
    flake-utils.url = "github:numtide/flake-utils";

    # rust vendor cache
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    # virtual machine images - assembling VM is simple and fast as OCI image :) 
    nixos-generators = {
      url = "github:nix-community/nixos-generators";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    # terraform generator to manage clouds/managed services
    terranix = {
      url = "github:terranix/terranix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    # fixed derivation for terraform packages
    nixpkgs-terraform-providers-bin = {
      url = "github:nix-community/nixpkgs-terraform-providers-bin";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    # for subkey
    substrate = {
      url = "github:dzmitry-lahoda-forks/substrate/8e8e54d99f5f86da1ff984646dc6cba3597a42f8";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  nixConfig = {
    # so you do not need to build locally if CI did it (no cache for ARM/MAC because did not added machines to build matrix)
    extra-substituters = [ "https://cache.nixos.org" "https://golden-gate-ggx.cachix.org" ];
    extra-trusted-public-keys = [ "cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY=" "golden-gate-ggx.cachix.org-1:h2zGCI9FqxUS7HxnZJDHaJzbN4iTsWvBcETdd+/0ZD4=" ];
  };

  # inputs and systems are know ahead of time -> we can evalute all nix -> flake make nix """statically typed"""
  outputs = { self, nixpkgs, devenv, rust-overlay, crane, flake-utils, terranix, nixos-generators, nixpkgs-terraform-providers-bin, substrate } @ inputs:
    let
      email = "<email>";
      domain = "ggchain.technology";
      org = "ggchaintesta";
      region = "eu-west-1";
      # could generate dynamic nixos module to be referenced
      bootnode = "34-244-81-67";
      bootnode-peer = "12D3KooWP3E64xQfgSdubAXpVrJTxL6a2Web2uiwC4PBxyEJFac3";
      # can use envvars override to allow run non shared "cloud" for tests
      age-pub = "age1a8k02z579lr0qr79pjhlneffjw3dvy3a8j5r4fw3zlphd6cyaf5qukkat5";

      per_system = flake-utils.lib.eachDefaultSystem
        (system:
          let

            overlays = [ (import rust-overlay) (substrate.overlays) ];
            pkgs = import nixpkgs {
              inherit system overlays;
            };

            # not optimal as not all packages requires this,
            # but many build.rs do - so we add little bit slowness for simplificaiton and reproduceability
            rust-native-build-inputs = with pkgs; [ clang pkg-config gnumake ];

            # reusable env for shell and builds
            rust-env = with pkgs; {
              LD_LIBRARY_PATH = pkgs.lib.strings.makeLibraryPath [
                pkgs.stdenv.cc.cc.lib
                pkgs.llvmPackages.libclang.lib
              ];
              LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
              PROTOC = "${pkgs.protobuf}/bin/protoc";
              ROCKSDB_LIB_DIR = "${pkgs.rocksdb}/lib";
              RUSTUP_TOOLCHAIN = (builtins.fromTOML (builtins.readFile ./rust-toolchain.toml)).toolchain.channel; # for dylint
            };

            darwin = pkgs.lib.optionals pkgs.stdenv.isDarwin (with pkgs.darwin.apple_sdk; [
              frameworks.Security
            ]);

            common-attrs = rust-env // {
              buildInputs = with pkgs; [ openssl zstd ];
              nativeBuildInputs = with pkgs;
                rust-native-build-inputs ++ [ openssl ] ++ darwin;
              doCheck = false;
              cargoCheckCommand = "true";
              src = rust-src;
              pname = "...";
              version = "...";
            };

            common-wasm-deps-attrs = common-attrs // {
              cargoExtraArgs =
                "--package 'golden-gate-runtime-*' --target wasm32-unknown-unknown --no-default-features --features=aura,with-rocksdb-weights";
              RUSTFLAGS =
                "-Clink-arg=--export=__heap_base -Clink-arg=--import-memory";
            };

            common-wasm-attrs = common-wasm-deps-attrs // {
              installPhase = ''
                runHook preInstall
                mkdir --parents $out/lib
                cp ./target/wasm32-unknown-unknown/release/wbuild/golden-gate-runtime-*/golden_gate_runtime_*.compact.compressed.wasm $out/lib
                runHook postInstall
              '';
            };


            # calls `cargo vendor` on package deps 
            common-wasm-deps =
              craneLib.buildDepsOnly (common-wasm-attrs // { });

            doclint = pkgs.writeShellApplication rec {
              name = "doclint";
              text = ''
                ${pkgs.lib.meta.getExe pkgs.nodePackages.markdownlint-cli2} "**/*.md" "#.devenv" "#target" "#terraform" "#result"
              '';
            };

            fix = pkgs.writeShellApplication rec {
              name = "fix";
              runtimeInputs = [
                rust-toolchain
              ];
              text = ''
                cargo clippy --fix --allow-staged --allow-dirty
                cargo fmt                
              '';
            };


            # rust used by ci and developers
            rust-toolchain =
              pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
            craneLib = (crane.mkLib pkgs).overrideToolchain rust-toolchain;

            # do not consier target to be part of source
            rust-src = pkgs.lib.cleanSourceWith {
              src = pkgs.lib.cleanSource ./.;
              filter = pkgs.nix-gitignore.gitignoreFilterPure
                (name: type:
                  # nix files are not used as part of build
                  (
                    (type == "regular" && pkgs.lib.strings.hasSuffix ".nix" name)
                    == false
                    &&
                    (type == "directory" && ".github" == name) == false
                    && (type == "directory" && "terraform" == name) == false

                    # risky, until we move code into separate repo as rust can do include_str! as doc, but good optimization
                    && (type == "regular" && pkgs.lib.strings.hasSuffix ".md" name) == false
                    && (type == "regular" && pkgs.lib.strings.hasSuffix ".json" name) == false
                    && (type == "regular" && pkgs.lib.strings.hasSuffix ".gitignore" name) == false
                  )
                )
                [ ./.gitignore ] ./.;
            };

            common-native-release-attrs = common-attrs // rec {
              cargoExtraArgs = "--package ${pname}";
              pname = "golden-gate-node";
              version = "0.1.0";
            };

            common-native-release-deps =
              craneLib.buildDepsOnly (common-native-release-attrs // { });
            common-wasm-release-deps = craneLib.buildDepsOnly common-wasm-deps-attrs;

            golden-gate-runtimes = craneLib.buildPackage (common-wasm-attrs // rec {
              pname = "golden-gate-runtimes";
              cargoArtifacts = common-wasm-release-deps;
            });

            golden-gate-node = craneLib.buildPackage (common-native-release-attrs // {
              cargoArtifacts = common-native-release-deps;
              nativeBuildInputs = common-native-release-attrs.nativeBuildInputs ++ [ pkgs.git ]; # parity does some git hacks in build.rs 
            });

            fmt = craneLib.cargoFmt (common-attrs // {
              cargoExtraArgs = "--all";
              rustFmtExtraArgs = "--color always";
            });

            cargoClippyExtraArgs = "-- -D warnings";

            clippy-node = craneLib.cargoClippy (common-native-release-attrs // {
              inherit cargoClippyExtraArgs;
              cargoArtifacts = golden-gate-node.cargoArtifacts;
            });

            clippy-wasm = craneLib.cargoClippy (common-wasm-deps-attrs // {
              inherit cargoClippyExtraArgs;
              cargoArtifacts = golden-gate-runtimes.cargoArtifacts;
            });



            # really need to run as some points:
            # - light client emulator (ideal for contracts)
            # - multi node local fast (fast druation low security)
            # - multi local slow (duration and security as in prod)
            # - here can apply above to remote with something if needed (terranix/terraform-ng works)
            # for each 
            # - either start from genesis
            # - of from fork (remote prod data)
            # all with - archieval and logging enabled
            single-fast = pkgs.writeShellApplication rec {
              name = "single-fast";
              text = ''
                ${pkgs.lib.meta.getExe golden-gate-node} --dev  
              '';
            };

            # we do not use existing Dotsama tools as they target relay + parachains
            # here we can evolve into generating arion/systemd/podman/k8s output (what ever will fit) easy 
            multi-fast = pkgs.writeShellApplication rec {
              name = "multi-fast";
              text = ''
                WS_PORT_ALICE=''${WS_PORT_ALICE:-9944}
                WS_PORT_BOB=''${WS_PORT_BOB:-9945}
                WS_PORT_CHARLIE=''${WS_PORT_CHARLIE:-9946}
                ( ${pkgs.lib.meta.getExe golden-gate-node} --chain=local --rpc-cors=all --alice --tmp --ws-port="$WS_PORT_ALICE" &> alice.log ) &
                ( ${pkgs.lib.meta.getExe golden-gate-node} --chain=local --rpc-cors=all --bob --tmp --ws-port="$WS_PORT_BOB" &> bob.log ) &
                ( ${pkgs.lib.meta.getExe golden-gate-node} --chain=local --rpc-cors=all --charlie --tmp --ws-port="$WS_PORT_CHARLIE" &> charlie.log ) &
                echo https://polkadot.js.org/apps/?rpc=ws://127.0.0.1:"$WS_PORT_ALICE"#/explorer
              '';
            };

            tf-init = pkgs.writeShellApplication rec {
              name = "tf-init";
              text = ''
                # here you manually obtain login key
                aws configure
              '';
            };

            cloud-tools = with pkgs; [
              awscli2
              terraform
              sops
              age
              nixos-rebuild
            ];

            # seldom to change node image to terraform onto cloud as template
            node-image = nixos-generators.nixosGenerate {
              system = "x86_64-linux";
              modules = [
                ./flake/nixos-amazon.nix
              ] ++ [ ({ ... }: { amazonImage.sizeMB = 16 * 1024; }) ]
              ;
              format = "amazon";
            };

            # send variables to terraform
            terraformattrs = {
              TF_VAR_VALIDATOR_NAME = org;
              TF_VAR_AWS_REGION = region;
            };

            terraformtestnet = terraformattrs // {
              TF_VAR_DOMAIN_NAME = domain;
            };

            terraformbase = terraformattrs // {
              TF_VAR_NODE_IMAGE = "\"$(find ${node-image} -type f -name '*.vhd')\"";
            };


            # generates node secrtekey and gets public key of
            gen-node-key = pkgs.writeShellApplication {
              name = "gen-node-key";
              runtimeInputs = [ pkgs.subkey pkgs.jq ];
              text = ''
                KEY_PATH=$1
                subkey generate --output-type=json --scheme=sr25519 > "$KEY_PATH".sr25519.json
                subkey inspect --scheme ed25519 --output-type=json "$(jq --raw-output .secretSeed "$KEY_PATH".sr25519.json)" > "$KEY_PATH".ed25519.json 

                echo "Address/AuthorityId/IAmOnline/Aura:"
                jq --raw-output .ss58Address "$KEY_PATH".sr25519.json
                
                echo "GRANDPA:"
                jq --raw-output .ss58Address "$KEY_PATH".ed25519.json

                echo "Peer identity (libp2p):"
                jq --raw-output .secretSeed "$KEY_PATH".ed25519.json |  subkey inspect-node-key
              '';
            };

            inspect-node-key = pkgs.writeShellApplication {
              name = "inspect-node-key";
              runtimeInputs = [ pkgs.subkey pkgs.jq ];
              text = ''
                KEY_PATH=$1
                echo "Address/AuthorityId/IAmOnline/Aura:"
                jq --raw-output .ss58Address "$KEY_PATH".sr25519.json
                
                echo "GRANDPA:"
                jq --raw-output .ss58Address "$KEY_PATH".ed25519.json

                echo "Peer identity (libp2p):"
                jq --raw-output .secretSeed "$KEY_PATH".ed25519.json |  subkey inspect-node-key
              '';
            };

            # need to generalize for generic TF_VAR consumption
            mkTerraformRun = tfname: config: attrs: pkgs.writeShellApplication (rec  {
              name = "
                tf-${tfname}";
              runtimeInputs = cloud-tools;
              text =
                # just way to put attrs as export variables
                with builtins; concatStringsSep "\n" (attrValues (mapAttrs (name: value: "${name}=${value} && export ${name}") attrs)) +
                  ''
                
                cd ./terraform/${tfname}  
                # generate terraform input from nix
                cp --force ${config} config-${tfname}.tf.json
            
                # silly check to avoid providers rechek all the time (nixified version would be more robust)
                if [[ ! -d .terraform/providers ]]; then
                  terraform init --upgrade
                fi
            
                # decrypt secret state (should run only on CI eventually for safety)
                # if there is encrypted state, decrypt it
                if [[ -f terraform-${tfname}.tfstate.sops ]]; then
                  # uses age, so can use any of many providers (including aws)
                  echo "decrypting state"
                  sops --decrypt --age ${age-pub} terraform-${tfname}.tfstate.sops > terraform-${tfname}.tfstate
                  # testing that we can finally reencrypt          
                  sops --encrypt --age ${age-pub} terraform-${tfname}.tfstate > terraform-${tfname}.tfstate.sops   
                fi
            
                # so we can store part of changes before exit
                set +o errexit
                # apply state to cloud, eventually should manually approve in CI
                terraform "$@" # for example `-- apply -auto-approve`
                TERRAFORM_RESULT=$?
                set -o errexit
              
                # encrypt update state back and push it (later in CI special job)
                echo "encrypting current state"
                if [[ -f terraform-${tfname}.tfstate ]]; then
                  sops --encrypt --age ${age-pub} terraform-${tfname}.tfstate > terraform-${tfname}.tfstate.sops
                fi

                if [[ -f terraform-${tfname}.tfstate.backup ]]; then 
                  echo "encrypting backup state"
                  sops --encrypt --age ${age-pub} terraform-${tfname}.tfstate.backup > terraform-${tfname}.tfstate.backup.sops
                fi
              
                exit $TERRAFORM_RESULT
              '';
            });

            tf-testnet = mkTerraformRun "testnet" tf-config-testnet terraformtestnet;
            tf-base = mkTerraformRun "base" tf-config-base terraformbase;


            tf-config-base = terranix.lib.terranixConfiguration {
              inherit system;
              modules = [ ./flake/terraform/base.nix ];
            };
            tf-config-testnet = terranix.lib.terranixConfiguration {
              inherit system;
              modules = [ ./flake/terraform/testnet.nix ];
            };

            mkNixosAwsRemoteRebuild = ip: region: name: pkgs.writeShellApplication
              {
                name = "deploy-" + name;
                runtimeInputs = [ pkgs.nixos-rebuild ];
                text = ''
                  # builds node locally and delta copies nix store to remote machine, and applies nix config
                  # should read from tfstate here to avoid cp paste of name       
                  NIX_SSHOPTS="-i ./terraform/testnet/id_rsa.pem"         
                  export NIX_SSHOPTS
                  # first run will be slow, so can consider variouse optimization later
                  nixos-rebuild switch --fast --flake .#${name}  --target-host root@ec2-${ip}.${region}.compute.amazonaws.com
                '';
              };
          in
          rec {

            packages = flake-utils.lib.flattenTree
              rec  {
                inherit fix golden-gate-runtimes golden-gate-node gen-node-key single-fast multi-fast tf-base tf-testnet node-image inspect-node-key doclint fmt clippy-node clippy-wasm;
                subkey = pkgs.subkey;
                node = golden-gate-node;
                lint-all = pkgs.symlinkJoin {
                  name = "lint-all";
                  paths = [ doclint fmt clippy-node clippy-wasm ];
                };
                release = pkgs.symlinkJoin {
                  name = "release";
                  paths = [ node golden-gate-runtimes ];
                };
                default = release;
                # we should prune 3 things:
                # - running process
                # - logs/storages of run proccess
                # - system prunce of nix cache/oci images
                prune-running = pkgs.writeShellApplication rec {
                  name = "prune-running";
                  text = ''
                    pkill golden-gate-nod 
                  '';
                };


                check-node = pkgs.writeShellApplication
                  {
                    name = "check-node";
                    text = ''
                      export SKIP_WASM_BUILD=1 && cargo check --package golden-gate-node
                    '';
                  };

                deploy-testnet-node-a = mkNixosAwsRemoteRebuild bootnode region "testnet-node-a";
                deploy-testnet-node-b = mkNixosAwsRemoteRebuild "34-243-72-53" region "testnet-node-b";
                deploy-testnet-node-c = mkNixosAwsRemoteRebuild "54-246-50-70" region "testnet-node-c";
                deploy-testnet-node-d = mkNixosAwsRemoteRebuild "3-253-35-79" region "testnet-node-d";

                run-testnet-node-a = pkgs.writeShellApplication {
                  name = "run-testnet-node-a";
                  runtimeInputs = [ pkgs.subkey pkgs.jq golden-gate-node ];
                  text = ''

                    RUST_LOG=info,libp2p=info,grandpa=info
                    export RUST_LOG
                    NODE_KEY=$(jq --raw-output .secretSeed /root/ed25519.json)
                    rm -r -f chains
                    
                    golden-gate-node key insert \
                      --base-path=/root/ \
                      --chain=testnet \
                      --scheme=sr25519 \
                      --suri "$NODE_KEY" \
                      --key-type aura \
                      --keystore-path ~/chains/remote_testnet/keystore

                    golden-gate-node key insert \
                      --base-path=/root/ \
                      --chain=testnet \
                      --scheme ed25519 \
                      --suri "$NODE_KEY" \
                      --key-type gran \
                      --keystore-path ~/chains/remote_testnet/keystore  

                    golden-gate-node --node-key "$NODE_KEY" --unsafe-ws-external --validator --rpc-methods=unsafe --unsafe-rpc-external --rpc-cors=all --blocks-pruning archive  --chain=testnet --name=node-a --base-path=/root/ 
                  '';
                };

                run-testnet-node-b = pkgs.writeShellApplication {
                  name = "run-testnet-node-b";
                  runtimeInputs = [ pkgs.subkey pkgs.jq golden-gate-node ];
                  text = ''
                    RUST_LOG=info,libp2p=info,grandpa=info
                    export RUST_LOG
                    NODE_KEY=$(jq --raw-output .secretSeed /root/ed25519.json)
                    rm -r -f /root/chains
                    
                    golden-gate-node key insert \
                      --base-path=/root/ \
                      --chain=testnet \
                      --scheme=sr25519 \
                      --suri "$NODE_KEY" \
                      --key-type aura \
                      --keystore-path ~/chains/remote_testnet/keystore

                    golden-gate-node key insert \
                      --base-path=/root/ \
                      --chain=testnet \
                      --scheme ed25519 \
                      --suri "$NODE_KEY" \
                      --key-type gran \
                      --keystore-path ~/chains/remote_testnet/keystore  

                    golden-gate-node --node-key "$NODE_KEY" --unsafe-ws-external --validator --rpc-methods=unsafe --unsafe-rpc-external --rpc-cors=all --blocks-pruning archive  --chain=testnet --name=node-b --base-path=/root/ --bootnodes=/ip4/34.244.81.67/tcp/30333/p2p/${bootnode-peer}
                  '';
                };

                run-testnet-node-c = pkgs.writeShellApplication {
                  name = "run-testnet-node-c";
                  runtimeInputs = [ pkgs.subkey pkgs.jq golden-gate-node ];
                  text = ''
                    RUST_LOG=info,libp2p=info,grandpa=info
                    export RUST_LOG
                    NODE_KEY=$(jq --raw-output .secretSeed /root/ed25519.json)
                    rm -r -f /root/chains
                    
                    golden-gate-node key insert \
                      --base-path=/root/ \
                      --chain=testnet \
                      --scheme=sr25519 \
                      --suri "$NODE_KEY" \
                      --key-type aura \
                      --keystore-path ~/chains/remote_testnet/keystore

                    golden-gate-node key insert \
                      --base-path=/root/ \
                      --chain=testnet \
                      --scheme ed25519 \
                      --suri "$NODE_KEY" \
                      --key-type gran \
                      --keystore-path ~/chains/remote_testnet/keystore  

                    golden-gate-node --node-key "$NODE_KEY" --unsafe-ws-external --validator --rpc-methods=unsafe --unsafe-rpc-external --rpc-cors=all --blocks-pruning archive  --chain=testnet --name=node-c --base-path=/root/ --bootnodes=/ip4/34.244.81.67/tcp/30333/p2p/${bootnode-peer}
                  '';
                };
                run-testnet-node-d = pkgs.writeShellApplication {
                  name = "run-testnet-node-d";
                  runtimeInputs = [ pkgs.subkey pkgs.jq golden-gate-node ];
                  text = ''
                    RUST_LOG=info,libp2p=info,grandpa=info
                    export RUST_LOG
                    NODE_KEY=$(jq --raw-output .secretSeed /root/ed25519.json)
                    rm -r -f /root/chains
                    
                    golden-gate-node key insert \
                      --base-path=/root/ \
                      --chain=testnet \
                      --scheme=sr25519 \
                      --suri "$NODE_KEY" \
                      --key-type aura \
                      --keystore-path ~/chains/remote_testnet/keystore

                    golden-gate-node key insert \
                      --base-path=/root/ \
                      --chain=testnet \
                      --scheme ed25519 \
                      --suri "$NODE_KEY" \
                      --key-type gran \
                      --keystore-path ~/chains/remote_testnet/keystore  

                    golden-gate-node --node-key "$NODE_KEY" --unsafe-ws-external --validator --rpc-methods=unsafe --unsafe-rpc-external --rpc-cors=all --blocks-pruning archive  --chain=testnet --name=node-d --base-path=/root/ --bootnodes=/ip4/34.244.81.67/tcp/30333/p2p/${bootnode-peer}
                  '';
                };
              };

            devShells = {
              default = devenv.lib.mkShell {
                inherit inputs pkgs;
                modules =
                  let

                    dylib = {
                      buildInputs = with pkgs; [ openssl ] ++ darwin;
                      nativeBuildInputs = rust-native-build-inputs;
                      doCheck = false;
                    };
                    rust-deps = pkgs.makeRustPlatform {
                      inherit pkgs;
                      # dylint needs nightly
                      cargo = pkgs.rust-bin.beta.latest.default;
                      rustc = pkgs.rust-bin.beta.latest.default;
                    };
                    cargo-dylint = with pkgs; rust-deps.buildRustPackage (rec {
                      pname = "cargo-dylint";
                      version = "2.1.5";
                      src = fetchCrate {
                        inherit pname version;
                        sha256 = "sha256-kH6dhUFaQpQ0kvzNyLIXjFAO8VNa2jah6ZaDO7LQKO0=";
                      };

                      cargoHash = "sha256-YvQI3H/4eWe6r2Tg8qHJqfnw/NpuGHtkRuTL4EzF0xo=";
                      cargoDepsName = pname;
                    } // dylib);
                    dylint-link = with pkgs; rust-deps.buildRustPackage (rec {
                      pname = "dylint-link";
                      version = "2.1.5";
                      src = fetchCrate {
                        inherit pname version;
                        sha256 = "sha256-oarEYhv0i2wAPmahx0vgWN3kmfEsK3s6D3+qkOqF9pc=";
                      };

                      cargoHash = "sha256-pMr9hddHAIyIclHRpxqdUaHphjSAVDnvfNjWGDA2EM4=";
                      cargoDepsName = pname;
                    } // dylib);
                    # can `cargo-contract` and nodejs ui easy here 
                  in
                  [
                    {
                      packages = with pkgs;
                        [
                          rust-toolchain
                          binaryen
                          llvmPackages.bintools
                          dylint-link
                          nodejs-18_x
                          nodePackages.markdownlint-cli2
                          jq
                          subkey
                        ]
                        ++ rust-native-build-inputs ++ darwin ++ cloud-tools;
                      env = rust-env;
                      # can do systemd/docker stuff here
                      enterShell = ''
                        echo ggshell
                      '';

                      # GH Codespace easy to run (e.g. for Mac users, low spec machines or Frontend developers or hackatons)
                      devcontainer.enable = true;
                    }
                  ];
              };
            };
          }
        );
    in
    per_system // {
      nixosConfigurations =
        let
          # nixos config for one and only one system
          # so it is invere of packages (packages for all systems)
          system = "x86_64-linux";
          overlays = [
            (import rust-overlay)
            (_: _: {
              golden-gate-node = per_system.packages.${system}.golden-gate-node;
            })
          ];
          pkgs = import nixpkgs {
            inherit system overlays;
          };
        in
        {
          # so basically cp pasted config to remote node with node binry
          # really should generate config after terraform run and load it dynamically
          testnet-node-a = let name = "node-a"; in nixpkgs.lib.nixosSystem {
            inherit system;
            modules = [
              {
                nixpkgs.overlays = [
                  (_: _: {
                    golden-gate-node = pkgs.golden-gate-node;
                  })
                ];
              }
              ./flake/web3nix-module.nix
              ./flake/nixos-amazon.nix
            ]
            ++ [
              ({ ... }: {
                web3nix.admin.email = email;
                services.nginx.virtualHosts = {
                  "${name}.${domain}" = {
                    addSSL = true;
                    enableACME = true;
                    root = "/var/www/default";
                    # just stub for root page, can route to any usefull info or landing
                    locations."/" = {
                      root = pkgs.runCommand "testdir" { } ''
                        mkdir "$out"
                        echo "here could be golden gate pwa" > "$out/index.html"
                      '';
                    };
                    locations."/substrate/client" = {
                      # any all to external servers is routed to node
                      proxyPass = "http://127.0.0.1:${builtins.toString 9944}";
                      proxyWebsockets = true;
                    };
                  };
                };
                security = {
                  acme = {
                    defaults.email = email;
                    acceptTerms = true;
                  };
                };
                environment.systemPackages = [ pkgs.golden-gate-node ];
                systemd.services.golden-gate-node = {
                  wantedBy = [ "multi-user.target" ];
                  after = [ "network.target" ];
                  description = "substrate-node";
                  serviceConfig =
                    {
                      Type = "simple";
                      User = "root";
                      # yeah, tune each unsafe on release
                      ExecStart = "${pkgs.lib.meta.getExe per_system.packages.${system}.run-testnet-node-a}";
                      Restart = "always";
                    };
                };

              })
            ];
          };

          testnet-node-b = let name = "node-b"; in nixpkgs.lib.nixosSystem {
            inherit system;
            modules = [
              {
                nixpkgs.overlays = [
                  (_: _: {
                    golden-gate-node = pkgs.golden-gate-node;
                  })
                ];
              }
              ./flake/web3nix-module.nix
              ./flake/nixos-amazon.nix
            ]
            ++ [
              ({ ... }: {
                web3nix.admin.email = email;
                services.nginx.virtualHosts = {
                  "${name}.${domain}" = {
                    addSSL = true;
                    enableACME = true;
                    root = "/var/www/default";
                    # just stub for root page, can route to any usefull info or landing
                    locations."/" = {
                      root = pkgs.runCommand "testdir" { } ''
                        mkdir "$out"
                        echo "here could be golden gate pwa" > "$out/index.html"
                      '';
                    };
                    locations."/substrate/client" = {
                      # any all to external servers is routed to node
                      proxyPass = "http://127.0.0.1:${builtins.toString 9944}";
                      proxyWebsockets = true;
                    };
                  };
                };
                security = {
                  acme = {
                    defaults.email = email;
                    acceptTerms = true;
                  };
                };
                environment.systemPackages = [ pkgs.golden-gate-node ];
                systemd.services.golden-gate-node =
                  {
                    wantedBy = [ "multi-user.target" ];
                    after = [ "network.target" ];
                    description = "substrate-node";
                    serviceConfig = {
                      Type = "simple";
                      User = "root";
                      ExecStart = "${pkgs.lib.meta.getExe per_system.packages.${system}.run-testnet-node-b}";
                      Restart = "always";
                    };
                  };

              })
            ];
          };

          testnet-node-c = let name = "node-c"; in nixpkgs.lib.nixosSystem {
            inherit system;
            modules = [
              {
                nixpkgs.overlays = [
                  (_: _: {
                    golden-gate-node = pkgs.golden-gate-node;
                  })
                ];
              }
              ./flake/web3nix-module.nix
              ./flake/nixos-amazon.nix
            ]
            ++ [
              ({ ... }: {
                web3nix.admin.email = email;
                services.nginx.virtualHosts = {
                  "${name}.${domain}" = {
                    addSSL = true;
                    enableACME = true;
                    root = "/var/www/default";
                    # just stub for root page, can route to any usefull info or landing
                    locations."/" = {
                      root = pkgs.runCommand "testdir" { } ''
                        mkdir "$out"
                        echo "here could be golden gate pwa" > "$out/index.html"
                      '';
                    };
                    locations."/substrate/client" = {
                      # any all to external servers is routed to node
                      proxyPass = "http://127.0.0.1:${builtins.toString 9944}";
                      proxyWebsockets = true;
                    };
                  };
                };
                security = {
                  acme = {
                    defaults.email = email;
                    acceptTerms = true;
                  };
                };
                environment.systemPackages = [ pkgs.golden-gate-node ];
                systemd.services.golden-gate-node =
                  {
                    wantedBy = [ "multi-user.target" ];
                    after = [ "network.target" ];
                    description = "substrate-node";
                    serviceConfig = {
                      Type = "simple";
                      User = "root";
                      ExecStart = "${pkgs.lib.meta.getExe per_system.packages.${system}.run-testnet-node-c}";
                      Restart = "always";
                    };
                  };

              })
            ];
          };

          testnet-node-d = let name = "node-d"; in nixpkgs.lib.nixosSystem {
            inherit system;
            modules = [
              {
                nixpkgs.overlays = [
                  (_: _: {
                    golden-gate-node = pkgs.golden-gate-node;
                  })
                ];
              }
              ./flake/web3nix-module.nix
              ./flake/nixos-amazon.nix
            ]
            ++ [
              ({ ... }: {
                web3nix.admin.email = email;
                services.nginx.virtualHosts = {
                  "${name}.${domain}" = {
                    addSSL = true;
                    enableACME = true;
                    root = "/var/www/default";
                    # just stub for root page, can route to any usefull info or landing
                    locations."/" = {
                      root = pkgs.runCommand "testdir" { } ''
                        mkdir "$out"
                        echo "here could be golden gate pwa" > "$out/index.html"
                      '';
                    };
                    locations."/substrate/client" = {
                      # any all to external servers is routed to node
                      proxyPass = "http://127.0.0.1:${builtins.toString 9944}";
                      proxyWebsockets = true;
                    };
                  };
                };
                security = {
                  acme = {
                    defaults.email = email;
                    acceptTerms = true;
                  };
                };
                environment.systemPackages = [ pkgs.golden-gate-node ];
                systemd.services.golden-gate-node =
                  {
                    wantedBy = [ "multi-user.target" ];
                    after = [ "network.target" ];
                    description = "substrate-node";
                    serviceConfig = {
                      Type = "simple";
                      User = "root";
                      ExecStart = "${pkgs.lib.meta.getExe per_system.packages.${system}.run-testnet-node-d}";
                      Restart = "always";
                    };
                  };

              })
            ];
          };
        };
    };
}

