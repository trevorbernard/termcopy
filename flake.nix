{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05-small";
  };

  outputs = {
    self,
    nixpkgs,
  }: let
    supportedSystems = [
      "x86_64-linux"
      "aarch64-darwin"
    ];
    forEachSupportedSystem = f:
      nixpkgs.lib.genAttrs supportedSystems (
        system:
          f {
            pkgs = import nixpkgs {
              inherit system;
            };
          }
      );
  in {
    formatter = forEachSupportedSystem ({pkgs}: pkgs.alejandra);

    packages = forEachSupportedSystem (
      {pkgs}:
        {
          default = pkgs.callPackage ./default.nix {};
        }
        // pkgs.lib.optionalAttrs pkgs.stdenv.isLinux {
          dockerImage = pkgs.dockerTools.buildLayeredImage {
            name = "termcopy";
            tag = "latest";
            contents = [(pkgs.callPackage ./default.nix {})];
            config = {
              Entrypoint = ["/bin/termcopy"];
            };
          };
        }
    );

    devShells = forEachSupportedSystem (
      {pkgs}: {
        default = pkgs.mkShell {
          nativeBuildInputs = [
            pkgs.cargo
            pkgs.cargo-audit
            pkgs.cargo-nextest
            pkgs.clippy
            pkgs.coreutils # for sha256sum
            pkgs.just
            pkgs.pkg-config
            pkgs.rustc
            pkgs.rust-analyzer
            pkgs.rustfmt
          ];
          buildInputs = [];
          shellHook = ''
            echo "Rust $(rustc --version)"
            echo "Cargo $(cargo --version)"
          '';
        };
      }
    );
  };
}
