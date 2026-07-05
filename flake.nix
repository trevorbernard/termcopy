{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05-small";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs =
    {
      nixpkgs,
      rust-overlay,
      ...
    }:
    let
      supportedSystems = [
        "x86_64-linux"
        "aarch64-darwin"
      ];
      forEachSupportedSystem =
        f:
        nixpkgs.lib.genAttrs supportedSystems (
          system:
          f {
            pkgs = import nixpkgs {
              inherit system;
              overlays = [ rust-overlay.overlays.default ];
            };
          }
        );
    in
    {
      formatter = forEachSupportedSystem ({ pkgs }: pkgs.nixfmt);

      packages = forEachSupportedSystem (
        { pkgs }:
        let
          rust = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
          rustPlatform = pkgs.makeRustPlatform {
            cargo = rust;
            rustc = rust;
          };
          default = pkgs.callPackage ./default.nix { inherit rustPlatform; };
        in
        {
          inherit default;
        }
        // pkgs.lib.optionalAttrs pkgs.stdenv.isLinux {
          dockerImage = pkgs.dockerTools.buildLayeredImage {
            name = "termcopy";
            tag = "latest";
            contents = [ default ];
            config = {
              Entrypoint = [ "/bin/termcopy" ];
            };
          };
        }
      );

      devShells = forEachSupportedSystem (
        { pkgs }:
        let
          rust = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        in
        {
          default = pkgs.mkShell {
            nativeBuildInputs = [
              rust
              pkgs.cargo-audit
              pkgs.cargo-nextest
              pkgs.coreutils # for sha256sum
              pkgs.just
              pkgs.pkg-config
            ];
            shellHook = ''
              echo "Rust $(rustc --version)"
              echo "Cargo $(cargo --version)"
            '';
          };
        }
      );
    };
}
