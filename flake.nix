# Credit to https://github.com/srid/rust-nix-template/blob/master/flake.nix
{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = github:numtide/flake-utils;
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
        pkgs = nixpkgs.legacyPackages.${system};
        rust-toolchain = pkgs.symlinkJoin {
          name = "rust-toolchain";
          paths = with pkgs; [
            rustc
            cargo
            cargo-watch
            rust-analyzer
            rustfmt
          ];
        };
      in
      rec {
        # This builds the blog binary then runs it and collects the output. Once done it throws away the binary and
        # shoves the newly created static site into the result.
        packages.default = pkgs.rustPlatform.buildRustPackage {
          name = "blog";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          postBuild = ''
            ./target/*/release/blog
          '';
          installPhase = ''
            cp -r ./out $out
          '';
        };

        overlays.default = packages.default;
        # Rust dev environment
        devShells.default = pkgs.mkShell {
          shellHook = ''
            # For rust-analyzer 'hover' tooltips to work.
            export RUST_SRC_PATH=${pkgs.rustPlatform.rustLibSrc}
          '';
          nativeBuildInputs = [ rust-toolchain ];
        };
      });
}
