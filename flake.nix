{
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

  };

  outputs = { self, nixpkgs, devenv, rust-overlay, crane, flake-utils, ... } @ inputs:

    flake-utils.lib.eachDefaultSystem (system:
      let

        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rust-deps = with pkgs; [ clang ];

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
            rust-deps ++ [ openssl pkg-config ] ++ darwin;
          doCheck = false;
          cargoCheckCommand = "true";
          src = rust-src;
        };


        common-wasm-attrs = common-attrs // {
          cargoExtraArgs = "--package golden-gate-runtime --target wasm32-unknown-unknown --no-default-features --features=with-paritydb-weights";
          RUSTFLAGS =
            "-Clink-arg=--export=__heap_base -Clink-arg=--import-memory";
          pname = "golden-gate-runtime";
          version = "0.1.0";

        };

        # calls `cargo vendor` on package deps 
        common-wasm-deps =
          craneLib.buildDepsOnly (common-wasm-attrs // { });


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
              )
            )


            [ ./.gitignore ] ./.;
        };

        golden-gate-runtime = craneLib.buildPackage (common-wasm-attrs // {
          installPhase = ''
            mkdir --parents $out/lib
            cp ./target/wasm32-unknown-unknown/release/wbuild/golden-gate-runtime/golden_gate_runtime.compact.compressed.wasm $out/lib
          '';
          src = rust-src;
          cargoArtifacts = common-wasm-deps;
        });
      in
      rec {
        packages = flake-utils.lib.flattenTree {
          inherit golden-gate-runtime;
          default = golden-gate-runtime;
        };


        devShells = {
          default = devenv.lib.mkShell {
            inherit inputs pkgs;
            modules =
              let

                dylib = {
                  buildInputs = with pkgs; [ openssl ] ++ darwin;
                  nativeBuildInputs = with pkgs;[ pkg-config ];
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
                  packages = with pkgs;[ rust-toolchain binaryen clang llvmPackages.bintools dylint-link ] ++ darwin;
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
