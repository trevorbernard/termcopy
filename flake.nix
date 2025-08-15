{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05-small";
  };

  outputs =
    {
      self,
      nixpkgs,
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
            };
          }
        );
    in
    {
      formatter = forEachSupportedSystem ({ pkgs }: pkgs.alejandra);

      packages = forEachSupportedSystem (
        { pkgs }:
        {
          default = pkgs.callPackage ./default.nix { };
        }
      );

      devShells = forEachSupportedSystem (
        { pkgs }:
        {
          default = pkgs.mkShell {
            nativeBuildInputs = [
              pkgs.cargo
              pkgs.cargo-audit
              pkgs.pkg-config
              pkgs.rustc
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
