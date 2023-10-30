{
  # like Cargo.toml or package.json dependencies, but on meta level (tools to run mentined files)
  inputs = {
    # base packages
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-23.05";

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

    # for subkey
    substrate = {
      url = "github:dzmitry-lahoda-forks/substrate/8e8e54d99f5f86da1ff984646dc6cba3597a42f8";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  nixConfig = {
    # so you do not need to build locally if CI did it (no cache for ARM/MAC because did not added machines to build matrix)
    extra-substituters = [ "https://cache.nixos.org" "https://golden-gate-ggx.cachix.org" ];
    extra-trusted-public-keys = [ "cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY=" "golden-gate-ggx.cachix.org-1:Sh6MjTG5qxsQcFDUMlkkRdAbTwZza9JqaETba9VgjnI=" ];
  };

  # inputs and systems are know ahead of time -> we can evalute all nix -> flake make nix """statically typed"""
  outputs = { self, nixpkgs, devenv, rust-overlay, crane, flake-utils, substrate } @ inputs:
    let
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
              cargoCheckCommand = "true";
              doCheck = false;
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

            common-native-runtime-common-attrs = common-attrs // rec {
              version = "0.1.2";
              pname = "ggxchain-runtime-common";
              cargoExtraArgs = "--package runtime-common";
            };

            common-native-release-sydney-deps =
              craneLib.buildDepsOnly (common-native-sydney-attrs // { });
            common-native-release-brooklyn-deps =
              craneLib.buildDepsOnly (common-native-brooklyn-attrs // { });
            common-wasm-release-deps = craneLib.buildDepsOnly common-wasm-deps-attrs;
            common-native-runtime-common-deps =
              craneLib.buildDepsOnly common-native-runtime-common-attrs;

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

            ggxchain-runtime-common = craneLib.buildPackage (common-native-runtime-common-attrs // {
              cargoArtifacts = common-native-runtime-common-deps;
            });

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

            cargoClippyExtraArgs = "-- -D warnings";
          in
          rec {
            checks = {
              fmt = craneLib.cargoFmt (common-attrs // {
                cargoExtraArgs = "--all";
                rustFmtExtraArgs = "--color always";
              });

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

              clippy-runtime-common = craneLib.cargoClippy (common-native-runtime-common-attrs // {
                inherit cargoClippyExtraArgs;
                cargoArtifacts = ggxchain-runtime-common.cargoArtifacts;
              });

              nextest-brooklyn = craneLib.cargoNextest (common-native-brooklyn-attrs // {
                cargoArtifacts = ggxchain-node-brooklyn.cargoArtifacts;
                doCheck = true;
              });

              nextest-sydney = craneLib.cargoNextest (common-native-sydney-attrs // {
                cargoArtifacts = ggxchain-node-sydney.cargoArtifacts;
                doCheck = true;
              });

              nextest-runtime-common = craneLib.cargoNextest (common-native-runtime-common-attrs // {
                cargoArtifacts = ggxchain-runtime-common.cargoArtifacts;
                doCheck = true;
              });

              doclint = pkgs.writeShellApplication rec {
                name = "doclint";
                text = ''
                  ${pkgs.lib.meta.getExe pkgs.nodePackages.markdownlint-cli2} "**/*.md" "#.devenv" "#target" "#terraform" "#result"
                '';
              };
            };

            packages = flake-utils.lib.flattenTree
              rec  {
                inherit custom-spec-files fix ggxchain-runtimes ggxchain-node-brooklyn ggxchain-node-sydney ggxchain-runtime-common gen-node-key inspect-node-key;
                subkey = pkgs.subkey;
                ggxchain-node = ggxchain-node-brooklyn;
                node = ggxchain-node;
                
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
              };

            devShells = {
              default = craneLib.devShell (rust-env // {
                checks = self.checks.${system};
                packages = with pkgs; [
                  rust-toolchain
                  nodejs-18_x
                  nodePackages.markdownlint-cli2
                  jq
                ];

                inputsFrom = [
                  ggxchain-runtimes
                  ggxchain-node-brooklyn
                  ggxchain-node-sydney
                  ggxchain-runtime-common
                ];
                   
                enterShell = ''
                  echo ggxshell
                '';
                name = "ggxshell";
              });
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
        };
    };
  }

