{ pkgs }:
let
  cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
in
pkgs.rustPlatform.buildRustPackage {
  pname = cargoToml.package.name;
  version = cargoToml.package.version;
  src = pkgs.lib.cleanSource ./.;
  cargoLock = {
    lockFile = ./Cargo.lock;
  };
  nativeBuildInputs = [
    pkgs.pkg-config
  ];
  buildInputs = [ ];
  meta = with pkgs.lib; {
    description = "A utility program that enables clipboard copying using OSC52 escape sequences";
    homepage = "https://github.com/trevorbernard/termcopy";
    license = licenses.mit;
    maintainers = with maintainers; [ ];
  };
}
