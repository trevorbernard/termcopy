{
  pkgs,
  rustPlatform ? pkgs.rustPlatform,
}:
let
  cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
  fs = pkgs.lib.fileset;
in
rustPlatform.buildRustPackage {
  pname = cargoToml.package.name;
  version = cargoToml.package.version;
  src = fs.toSource {
    root = ./.;
    fileset = fs.unions [
      ./Cargo.toml
      ./Cargo.lock
      ./src
      ./tests
    ];
  };
  cargoLock = {
    lockFile = ./Cargo.lock;
  };
  nativeBuildInputs = [
    pkgs.pkg-config
  ];
  postInstall = ''
    ln -s $out/bin/termcopy $out/bin/tc
  '';
  meta = with pkgs.lib; {
    description = "A utility program that enables clipboard copying using OSC52 escape sequences";
    homepage = "https://github.com/trevorbernard/termcopy";
    license = licenses.mit;
    maintainers = [
      {
        github = "trevorbernard";
        name = "Trevor Bernard";
        email = "trevor.bernard@pm.me";
      }
    ];
  };
}
