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

    # terraform generator to manage clouds/managed services
    terranix = {
      url = "github:terranix/terranix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

  };

  nixConfig = {
    # so you do not need to build locally if CI did it (no cache for ARM/MAC because did not added machines to build matrix)
    extra-substituters = [ "https://cache.nixos.org" "https://golden-gate-ggx.cachix.org" ];
    extra-trusted-public-keys = [ "cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY=" "golden-gate-ggx.cachix.org-1:h2zGCI9FqxUS7HxnZJDHaJzbN4iTsWvBcETdd+/0ZD4=" ];
  };

  # inputs and systems are know ahead of time -> we can evalute all nix -> flake make nix """statically typed"""
  outputs = { self, nixpkgs, devenv, rust-overlay, crane, flake-utils, terranix, ... } @ inputs:
    flake-utils.lib.eachDefaultSystem (system:
      let

        overlays = [ (import rust-overlay) ];
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
          RUSTUP_TOOLCHAIN = "nightly-2022-12-20"; # could read from toml for dylint
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
        };


        common-wasm-attrs = common-attrs // rec {
          # really would could read it from Cargo.toml and reuse in here and in CI publish script as refactoring
          pname = "golden-gate-runtime";
          cargoExtraArgs = "--package ${pname} --target wasm32-unknown-unknown --no-default-features --features=aura,with-rocksdb-weights";
          RUSTFLAGS =
            "-Clink-arg=--export=__heap_base -Clink-arg=--import-memory";
          version = "0.1.0";
        };


        common-native-release-attrs = common-attrs // rec {
          cargoExtraArgs = "--package ${pname}";
          pname = "golden-gate-node";
          version = "0.1.0";
        };

        # calls `cargo vendor` on package deps 
        common-wasm-deps =
          craneLib.buildDepsOnly (common-wasm-attrs // { });
        common-native-release-deps =
          craneLib.buildDepsOnly (common-native-release-attrs // { });


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
              )
            )

            [ ./.gitignore ] ./.;
        };

        golden-gate-runtime = craneLib.buildPackage (common-wasm-attrs // {
          installPhase = ''
            mkdir --parents $out/lib
            cp ./target/wasm32-unknown-unknown/release/wbuild/${common-wasm-attrs.pname}/golden_gate_runtime.compact.compressed.wasm $out/lib
          '';
          src = rust-src;
          cargoArtifacts = common-wasm-deps;
        });

        golden-gate-node = craneLib.buildPackage (common-native-release-attrs // {
          src = rust-src;
          cargoArtifacts = common-native-release-deps;
          nativeBuildInputs = common-native-release-attrs.nativeBuildInputs ++ [ pkgs.git ]; # parity does some git hacks in build.rs 
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
            WS_PORT_ALICE=''${WS_PORT_ALICE:-9988}
            WS_PORT_BOB=''${WS_PORT_BOB:-9989}
            WS_PORT_CHARLIE=''${WS_PORT_CHARLIE:-9990}
            ( ${pkgs.lib.meta.getExe golden-gate-node} --chain=local --rpc-cors=all --alice --tmp --ws-port="$WS_PORT_ALICE" &> alice.log ) &
            ( ${pkgs.lib.meta.getExe golden-gate-node} --chain=local --rpc-cors=all --bob --tmp --ws-port="$WS_PORT_BOB" &> bob.log ) &
            ( ${pkgs.lib.meta.getExe golden-gate-node} --chain=local --rpc-cors=all --charlie --tmp --ws-port="$WS_PORT_CHARLIE" &> charlie.log ) &
            echo https://polkadot.js.org/apps/?rpc=ws://127.0.0.1:"$WS_PORT_ALICE"#/explorer
          '';
        };

        lint = pkgs.writeShellApplication rec {
          name = "lint";
          text = ''
            ${pkgs.lib.meta.getExe pkgs.nodePackages.markdownlint-cli2} "**/*.md" "#.devenv" "#target"
          '';
        };


        tf-init = pkgs.writeShellApplication rec {
          name = "tf-init";
          text = ''
            # here you manually obtain login key
            aws configure
          '';
        };

        # can use envvars override to allow run non shared "cloud" for tests
        age-pub = "age1a8k02z579lr0qr79pjhlneffjw3dvy3a8j5r4fw3zlphd6cyaf5qukkat5";
        cloud-tools = with pkgs; [
          awscli2
          terraform
          sops
          age
        ];
        tf-apply = pkgs.writeShellApplication rec {
          name = "tf-apply";
          runtimeInputs = cloud-tools;
          text = ''
            cd ./terraform  
            # generate terraform input from nix
            cp --force ${tf-config} config.tf.json
            terraform init --upgrade

            # decrypt secret state (should run only on CI eventually for safety)
            # if there is encrypted state, decrypt it
            if [[ -f terraform.tfstate.sops ]]; then
              # uses age, so can use any of many providers (including aws)
              sops --decrypt --age ${age-pub} terraform.tfstate.sops > terraform.tfstate          
            fi
          
            # apply state to cloud, eventually should manually approve in CI
            terraform apply -auto-approve
            # encrypt update state back and push it (later in CI special job)
            sops --encrypt --age ${age-pub} terraform.tfstate > terraform.tfstate.sops
            # seems good idea to encrypt backup here too
          '';
        };


        tf-config = terranix.lib.terranixConfiguration {
          inherit system;
          modules = [ ./flake/terraform.nix ];
        };
      in
      rec {
        packages = flake-utils.lib.flattenTree {
          inherit golden-gate-runtime golden-gate-node single-fast multi-fast tf-config tf-apply lint;
          node = golden-gate-node;
          runtime = golden-gate-runtime;
          default = golden-gate-runtime;
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
}
