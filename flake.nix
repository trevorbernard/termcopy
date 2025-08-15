{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05-small";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
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
              overlays = [
                rust-overlay.overlays.default
                self.overlays.default
              ];
            };
          }
        );
    in
    {
      overlays.default = final: prev: {
        rustToolchain = prev.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
      };

      formatter = forEachSupportedSystem ({ pkgs }: pkgs.alejandra);

      packages = forEachSupportedSystem (
        { pkgs }:
        {
          default = pkgs.callPackage ./default.nix {
            rustToolchain = pkgs.rustToolchain;
          };
        }
      );

      devShells = forEachSupportedSystem (
        { pkgs }:
        {
          default = pkgs.mkShell {
            nativeBuildInputs = [
              pkgs.cargo-audit
              pkgs.pkg-config
              pkgs.rustToolchain
            ];
            buildInputs = [ ];
            shellHook = ''
              echo "Rust $(rustc --version)"
              echo "Cargo $(cargo --version)"
            '';
          };
        }
      );
    };
}
