{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-22.11";
    devenv.url = "github:cachix/devenv";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, devenv, rust-overlay, ... } @ inputs:
    let
      systems = [ "x86_64-linux" "x86_64-darwin" "aarch64-linux" "aarch64-darwin" ];
      forAllSystems = f: builtins.listToAttrs (map (name: { inherit name; value = f name; }) systems);
    in
    {
      devShells = forAllSystems
        (system:
          let
            overlays = [ (import rust-overlay) ];
            pkgs = import nixpkgs {
              inherit system overlays;
            };
          in
          {
            default = devenv.lib.mkShell {
              inherit inputs pkgs;
              modules =
                let
                  rust-toolchain =
                    pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

                  cargo-dylint = with pkgs; rustPlatform.buildRustPackage rec {
                    pname = "cargo-dylint";
                    version = "2.1.5";
                    nativeBuildInputs = [ pkgs.pkg-config ];
                    src = fetchCrate {
                      inherit pname version;
                      sha256 = "sha256-kH6dhUFaQpQ0kvzNyLIXjFAO8VNa2jah6ZaDO7LQKO0=";
                    };

                    cargoHash = "sha256-YvQI3H/4eWe6r2Tg8qHJqfnw/NpuGHtkRuTL4EzF0xo=";
                    cargoDepsName = pname;
                  };
                  dylint-link = with pkgs; rustPlatform.buildRustPackage rec {
                    pname = "dylint-link";
                    version = "2.1.5";
                    nativeBuildInputs = [ pkgs.pkg-config ];
                    src = fetchCrate {
                      inherit pname version;
                      sha256 = "sha256-oarEYhv0i2wAPmahx0vgWN3kmfEsK3s6D3+qkOqF9pc=";
                    };

                    cargoHash = "sha256-YvQI3H/4eWe6r2Tg8qHJqfnw/NpuGHtk3uTL4EzF1xo=";
                    cargoDepsName = pname;
                  };

                in
                [
                  {
                    packages = with pkgs;[ rust-toolchain clang llvmPackages.bintools cargo-dylint dylint-link ] ++ lib.optionals pkgs.stdenv.isDarwin (with pkgs.darwin.apple_sdk; [
                      frameworks.Security
                    ]);

                    env = with pkgs; {
                      LD_LIBRARY_PATH = pkgs.lib.strings.makeLibraryPath [
                        pkgs.stdenv.cc.cc.lib
                        pkgs.llvmPackages.libclang.lib
                      ];
                      LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
                      PROTOC = "${pkgs.protobuf}/bin/protoc";
                      ROCKSDB_LIB_DIR = "${pkgs.rocksdb}/lib";
                    };
                    enterShell = ''
                      echo ggshell
                    '';
                  }
                ];
            };
          });
    };
}
