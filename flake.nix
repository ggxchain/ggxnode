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
    extra-trusted-public-keys = [ "cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY=" "ggx-ggx.cachix.org-1:Sh6MjTG5qxsQcFDUMlkkRdAbTwZza9JqaETba9VgjnI=" ];
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
                (pkgs.lib.makeLibraryPath [pkgs.openssl])
              ];
              OPENSSL_DIR = "${pkgs.openssl.dev}";
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
                "--package 'ggxchain-runtime-*' --target wasm32-unknown-unknown --no-default-features";
              RUSTFLAGS =
                "-Clink-arg=--export=__heap_base -Clink-arg=--import-memory";
            };

            common-wasm-attrs = common-wasm-deps-attrs // {
              installPhase = ''
                runHook preInstall
                mkdir --parents $out/lib
                cp ./target/wasm32-unknown-unknown/release/wbuild/ggxchain-runtime-*/ggxchain_runtime_*.compact.compressed.wasm $out/lib
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
              filter = let
                isNixFile = name: type:
                  type == "regular" && pkgs.lib.strings.hasSuffix ".nix" name;
                isGithubDir = name: type:
                  type == "directory" && ".github" == name;
                isTerraformDir = name: type:
                  type == "directory" && "terraform" == name;

                # risky, until we move code into separate repo as rust can do include_str! as doc, but good optimization
                isMarkdownFile = name: type:
                  type == "regular" && pkgs.lib.strings.hasSuffix ".md" name;
                isGitIgnoreFile = name: type:
                  type == "regular" && pkgs.lib.strings.hasSuffix ".gitignore" name;
                customFilter = name: type:
                  ! builtins.any (fun: fun name type) [
                    isNixFile
                    isGithubDir
                    isTerraformDir
                    isMarkdownFile
                  ];
              in pkgs.nix-gitignore.gitignoreFilterPure customFilter [ ./.gitignore ] ./.;
            };

            custom-spec-files = pkgs.stdenv.mkDerivation {
              name = "custom-spec-files";
              src = builtins.path { path = ./.; name = "custom-spec-files"; };
              installPhase = ''
                mkdir -p $out
                cp $src/custom-spec-files/* $out/
              '';
            };

            common-native-release-attrs = { runtime, version }:
              common-attrs // rec {
                  inherit version;
                  cargoExtraArgs = "--package ${pname} --no-default-features --features=${runtime}";
                  pname = "ggxchain-node";
                  nativeBuildInputs = common-attrs.nativeBuildInputs ++ [ pkgs.git ]; # parity does some git hacks in build.rs
              };

            common-native-sydney-attrs = common-native-release-attrs {
              runtime = "sydney";
              version = "0.1.2";
            };

            common-native-brooklyn-attrs = common-native-release-attrs {
              runtime = "brooklyn";
              version = "0.2.0";
            };

            common-native-release-sydney-deps =
              craneLib.buildDepsOnly (common-native-sydney-attrs // { });
            common-native-release-brooklyn-deps =
              craneLib.buildDepsOnly (common-native-brooklyn-attrs // { });
            common-wasm-release-deps = craneLib.buildDepsOnly common-wasm-deps-attrs;

            ggxchain-runtimes = craneLib.buildPackage (common-wasm-attrs // rec {
              pname = "ggxchain-runtimes";
              cargoArtifacts = common-wasm-release-deps;
            });

            ggxchain-node-sydney = craneLib.buildPackage (common-native-sydney-attrs // {
              cargoArtifacts = common-native-release-sydney-deps;
            });

            ggxchain-node-brooklyn = craneLib.buildPackage (common-native-brooklyn-attrs // {
              cargoArtifacts = common-native-release-brooklyn-deps;
            });

            fmt = craneLib.cargoFmt (common-attrs // {
              cargoExtraArgs = "--all";
              rustFmtExtraArgs = "--color always";
            });

            cargoClippyExtraArgs = "-- -D warnings";

            clippy-node-brooklyn = craneLib.cargoClippy (common-native-brooklyn-attrs // {
              inherit cargoClippyExtraArgs;
              cargoArtifacts = ggxchain-node-brooklyn.cargoArtifacts;
            });

            clippy-node-sydney = craneLib.cargoClippy (common-native-sydney-attrs // {
              inherit cargoClippyExtraArgs;
              cargoArtifacts = ggxchain-node-sydney.cargoArtifacts;
            });

            clippy-wasm = craneLib.cargoClippy (common-wasm-deps-attrs // {
              inherit cargoClippyExtraArgs;
              cargoArtifacts = ggxchain-runtimes.cargoArtifacts;
            });

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

            terraformbrooklyn = terraformattrs // {
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

            tf-brooklyn = mkTerraformRun "brooklyn" tf-config-brooklyn terraformbrooklyn;
            tf-base = mkTerraformRun "base" tf-config-base terraformbase;


            tf-config-base = terranix.lib.terranixConfiguration {
              inherit system;
              modules = [ ./flake/terraform/base.nix ];
            };
            tf-config-brooklyn = terranix.lib.terranixConfiguration {
              inherit system;
              modules = [ ./flake/terraform/brooklyn.nix ];
            };

            mkNixosAwsRemoteRebuild = ip: region: name: pkgs.writeShellApplication
              {
                name = "deploy-" + name;
                runtimeInputs = [ pkgs.nixos-rebuild ];
                text = ''
                  # builds node locally and delta copies nix store to remote machine, and applies nix config
                  # should read from tfstate here to avoid cp paste of name       
                  NIX_SSHOPTS="-i ./terraform/brooklyn/id_rsa.pem"         
                  export NIX_SSHOPTS
                  # first run will be slow, so can consider variouse optimization later
                  nixos-rebuild switch --fast --flake .#${name}  --target-host root@ec2-${ip}.${region}.compute.amazonaws.com
                '';
              };
          in
          rec {

            packages = flake-utils.lib.flattenTree
              rec  {
                inherit custom-spec-files fix ggxchain-runtimes ggxchain-node-brooklyn ggxchain-node-sydney gen-node-key tf-base tf-brooklyn node-image inspect-node-key doclint fmt clippy-node-brooklyn clippy-node-sydney clippy-wasm;
                subkey = pkgs.subkey;
                ggxchain-node = ggxchain-node-brooklyn;
                node = ggxchain-node;
                lint-all = pkgs.symlinkJoin {
                  name = "lint-all";
                  paths = [ doclint fmt clippy-node-brooklyn clippy-node-sydney clippy-wasm ];
                };
                release = pkgs.symlinkJoin {
                  name = "release";
                  paths = [ node ggxchain-runtimes ];
                };
                default = release;
                # we should prune 3 things:
                # - running process
                # - logs/storages of run proccess
                # - system prunce of nix cache/oci images
                prune-running = pkgs.writeShellApplication rec {
                  name = "prune-running";
                  text = ''
                    pkill ggxchain-node
                  '';
                };

                sydney-node = pkgs.writeShellApplication rec {
                  name = "sydney-node";
                  text = ''
                    ${pkgs.lib.meta.getExe ggxchain-node-sydney} --chain=sydney
                  '';
                };

                brooklyn-node = pkgs.writeShellApplication rec {
                  name = "brooklyn-node";
                  text = ''
                    ${pkgs.lib.meta.getExe ggxchain-node-brooklyn} --chain=${custom-spec-files}/brooklyn.json
                  '';
                };

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
                    package=${pkgs.lib.meta.getExe ggxchain-node-brooklyn}
                    if [[ ''${1:-""} = "sydney" ]]; then
                      package=${pkgs.lib.meta.getExe ggxchain-node-sydney}
                    fi
                    $package --dev  
                  '';
                };

                # we do not use existing Dotsama tools as they target relay + parachains
                # here we can evolve into generating arion/systemd/podman/k8s output (what ever will fit) easy 
                multi-fast = pkgs.writeShellApplication rec {
                  name = "multi-fast";
                  text = ''
                    package=${pkgs.lib.meta.getExe ggxchain-node-brooklyn}
                    if [[ ''${1:-""} = "sydney" ]]; then
                      package=${pkgs.lib.meta.getExe ggxchain-node-sydney}
                    fi

                    WS_PORT_ALICE=''${WS_PORT_ALICE:-9944}
                    WS_PORT_BOB=''${WS_PORT_BOB:-9945}
                    WS_PORT_CHARLIE=''${WS_PORT_CHARLIE:-9946}
                    ( $package --chain=local --rpc-cors=all --alice --tmp --ws-port="$WS_PORT_ALICE" &> alice.log ) &
                    ( $package --chain=local --rpc-cors=all --bob --tmp --ws-port="$WS_PORT_BOB" &> bob.log ) &
                    ( $package --chain=local --rpc-cors=all --charlie --tmp --ws-port="$WS_PORT_CHARLIE" &> charlie.log ) &

                    echo https://explorer.ggxchain.io/?rpc=ws://127.0.0.1:"$WS_PORT_ALICE"#/explorer
                  '';
                };

                deploy-brooklyn-node-a = mkNixosAwsRemoteRebuild bootnode region "brooklyn-node-a";
                deploy-brooklyn-node-b = mkNixosAwsRemoteRebuild "34-243-72-53" region "brooklyn-node-b";
                deploy-brooklyn-node-c = mkNixosAwsRemoteRebuild "54-246-50-70" region "brooklyn-node-c";
                deploy-brooklyn-node-d = mkNixosAwsRemoteRebuild "3-253-35-79" region "brooklyn-node-d";

                run-brooklyn-node-a = pkgs.writeShellApplication {
                  name = "run-brooklyn-node-a";
                  runtimeInputs = [ pkgs.subkey pkgs.jq ggxchain-node ];
                  text = ''

                    RUST_LOG=info,libp2p=info,grandpa=info
                    export RUST_LOG
                    NODE_KEY=$(jq --raw-output .secretSeed /root/ed25519.json)
                    rm -r -f chains
                    
                    ggxchain-node key insert \
                      --base-path=/root/ \
                      --chain=brooklyn \
                      --scheme=sr25519 \
                      --suri "$NODE_KEY" \
                      --key-type aura \
                      --keystore-path ~/chains/remote_brooklyn/keystore

                    ggxchain-node key insert \
                      --base-path=/root/ \
                      --chain=brooklyn \
                      --scheme ed25519 \
                      --suri "$NODE_KEY" \
                      --key-type gran \
                      --keystore-path ~/chains/remote_brooklyn/keystore  

                    ggxchain-node --node-key "$NODE_KEY" --unsafe-ws-external --validator --rpc-methods=unsafe --unsafe-rpc-external --rpc-cors=all --blocks-pruning archive  --chain=brooklyn --name=node-a --base-path=/root/ 
                  '';
                };

                run-brooklyn-node-b = pkgs.writeShellApplication {
                  name = "run-brooklyn-node-b";
                  runtimeInputs = [ pkgs.subkey pkgs.jq ggxchain-node ];
                  text = ''
                    RUST_LOG=info,libp2p=info,grandpa=info
                    export RUST_LOG
                    NODE_KEY=$(jq --raw-output .secretSeed /root/ed25519.json)
                    rm -r -f /root/chains
                    
                    ggxchain-node key insert \
                      --base-path=/root/ \
                      --chain=brooklyn \
                      --scheme=sr25519 \
                      --suri "$NODE_KEY" \
                      --key-type aura \
                      --keystore-path ~/chains/remote_brooklyn/keystore

                    ggxchain-node key insert \
                      --base-path=/root/ \
                      --chain=brooklyn \
                      --scheme ed25519 \
                      --suri "$NODE_KEY" \
                      --key-type gran \
                      --keystore-path ~/chains/remote_brooklyn/keystore  

                    ggxchain-node --node-key "$NODE_KEY" --unsafe-ws-external --validator --rpc-methods=unsafe --unsafe-rpc-external --rpc-cors=all --blocks-pruning archive  --chain=brooklyn --name=node-b --base-path=/root/ --bootnodes=/ip4/34.244.81.67/tcp/30333/p2p/${bootnode-peer}
                  '';
                };

                run-brooklyn-node-c = pkgs.writeShellApplication {
                  name = "run-brooklyn-node-c";
                  runtimeInputs = [ pkgs.subkey pkgs.jq ggxchain-node ];
                  text = ''
                    RUST_LOG=info,libp2p=info,grandpa=info
                    export RUST_LOG
                    NODE_KEY=$(jq --raw-output .secretSeed /root/ed25519.json)
                    rm -r -f /root/chains
                    
                    ggxchain-node key insert \
                      --base-path=/root/ \
                      --chain=brooklyn \
                      --scheme=sr25519 \
                      --suri "$NODE_KEY" \
                      --key-type aura \
                      --keystore-path ~/chains/remote_brooklyn/keystore

                    ggxchain-node key insert \
                      --base-path=/root/ \
                      --chain=brooklyn \
                      --scheme ed25519 \
                      --suri "$NODE_KEY" \
                      --key-type gran \
                      --keystore-path ~/chains/remote_brooklyn/keystore  

                    ggxchain-node --node-key "$NODE_KEY" --unsafe-ws-external --validator --rpc-methods=unsafe --unsafe-rpc-external --rpc-cors=all --blocks-pruning archive  --chain=brooklyn --name=node-c --base-path=/root/ --bootnodes=/ip4/34.244.81.67/tcp/30333/p2p/${bootnode-peer}
                  '';
                };
                run-brooklyn-node-d = pkgs.writeShellApplication {
                  name = "run-brooklyn-node-d";
                  runtimeInputs = [ pkgs.subkey pkgs.jq ggxchain-node ];
                  text = ''
                    RUST_LOG=info,libp2p=info,grandpa=info
                    export RUST_LOG
                    NODE_KEY=$(jq --raw-output .secretSeed /root/ed25519.json)
                    rm -r -f /root/chains
                    
                    ggxchain-node key insert \
                      --base-path=/root/ \
                      --chain=brooklyn \
                      --scheme=sr25519 \
                      --suri "$NODE_KEY" \
                      --key-type aura \
                      --keystore-path ~/chains/remote_brooklyn/keystore

                    ggxchain-node key insert \
                      --base-path=/root/ \
                      --chain=brooklyn \
                      --scheme ed25519 \
                      --suri "$NODE_KEY" \
                      --key-type gran \
                      --keystore-path ~/chains/remote_brooklyn/keystore  

                    ggxchain-node --node-key "$NODE_KEY" --unsafe-ws-external --validator --rpc-methods=unsafe --unsafe-rpc-external --rpc-cors=all --blocks-pruning archive  --chain=brooklyn --name=node-d --base-path=/root/ --bootnodes=/ip4/34.244.81.67/tcp/30333/p2p/${bootnode-peer}
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
                          openssl
                        ]
                        ++ rust-native-build-inputs ++ darwin ++ cloud-tools;
                      env = rust-env;
                      # can do systemd/docker stuff here
                      enterShell = ''
                        echo ggxshell
                      '';
                      name = "ggxshell";

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
              ggxchain-node = per_system.packages.${system}.ggxchain-node;
            })
          ];
          pkgs = import nixpkgs {
            inherit system overlays;
          };
        in
        {
          # so basically cp pasted config to remote node with node binry
          # really should generate config after terraform run and load it dynamically
          brooklyn-node-a = let name = "node-a"; in nixpkgs.lib.nixosSystem {
            inherit system;
            modules = [
              {
                nixpkgs.overlays = [
                  (_: _: {
                    ggxchain-node = pkgs.ggxchain-node;
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
                        echo "here could be ggx chain pwa" > "$out/index.html"
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
                environment.systemPackages = [ pkgs.ggxchain-node ];
                systemd.services.ggxchain-node = {
                  wantedBy = [ "multi-user.target" ];
                  after = [ "network.target" ];
                  description = "substrate-node";
                  serviceConfig =
                    {
                      Type = "simple";
                      User = "root";
                      # yeah, tune each unsafe on release
                      ExecStart = "${pkgs.lib.meta.getExe per_system.packages.${system}.run-brooklyn-node-a}";
                      Restart = "always";
                    };
                };

              })
            ];
          };

          brooklyn-node-b = let name = "node-b"; in nixpkgs.lib.nixosSystem {
            inherit system;
            modules = [
              {
                nixpkgs.overlays = [
                  (_: _: {
                    ggxchain-node = pkgs.ggxchain-node;
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
                        echo "here could be ggx chain pwa" > "$out/index.html"
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
                environment.systemPackages = [ pkgs.ggxchain-node ];
                systemd.services.ggxchain-node =
                  {
                    wantedBy = [ "multi-user.target" ];
                    after = [ "network.target" ];
                    description = "substrate-node";
                    serviceConfig = {
                      Type = "simple";
                      User = "root";
                      ExecStart = "${pkgs.lib.meta.getExe per_system.packages.${system}.run-brooklyn-node-b}";
                      Restart = "always";
                    };
                  };

              })
            ];
          };

          brooklyn-node-c = let name = "node-c"; in nixpkgs.lib.nixosSystem {
            inherit system;
            modules = [
              {
                nixpkgs.overlays = [
                  (_: _: {
                    ggxchain-node = pkgs.ggxchain-node;
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
                        echo "here could be ggx chain pwa" > "$out/index.html"
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
                environment.systemPackages = [ pkgs.ggxchain-node ];
                systemd.services.ggxchain-node =
                  {
                    wantedBy = [ "multi-user.target" ];
                    after = [ "network.target" ];
                    description = "substrate-node";
                    serviceConfig = {
                      Type = "simple";
                      User = "root";
                      ExecStart = "${pkgs.lib.meta.getExe per_system.packages.${system}.run-brooklyn-node-c}";
                      Restart = "always";
                    };
                  };

              })
            ];
          };

          brooklyn-node-d = let name = "node-d"; in nixpkgs.lib.nixosSystem {
            inherit system;
            modules = [
              {
                nixpkgs.overlays = [
                  (_: _: {
                    ggxchain-node = pkgs.ggxchain-node;
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
                        echo "here could be ggx chain pwa" > "$out/index.html"
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
                environment.systemPackages = [ pkgs.ggxchain-node ];
                systemd.services.ggxchain-node =
                  {
                    wantedBy = [ "multi-user.target" ];
                    after = [ "network.target" ];
                    description = "substrate-node";
                    serviceConfig = {
                      Type = "simple";
                      User = "root";
                      ExecStart = "${pkgs.lib.meta.getExe per_system.packages.${system}.run-brooklyn-node-d}";
                      Restart = "always";
                    };
                  };

              })
            ];
          };
        };
    };
}

