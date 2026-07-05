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
          let
            pkgs = import nixpkgs {
              inherit system;
              overlays = [ rust-overlay.overlays.default ];
            };
          in
          f {
            inherit pkgs;
            rust = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
          }
        );
    in
    {
      formatter = forEachSupportedSystem ({ pkgs, ... }: pkgs.nixfmt);

      packages = forEachSupportedSystem (
        { pkgs, rust }:
        let
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
              User = "65534:65534";
            };
          };
        }
      );

      devShells = forEachSupportedSystem (
        { pkgs, rust }:
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
