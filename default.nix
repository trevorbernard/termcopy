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
  # With rust-src in the sysroot, rustc embeds absolute /nix/store paths to std
  # sources in panic-location strings, pulling the entire ~2GB toolchain into
  # the runtime closure (and the docker image). Remap them to a neutral prefix.
  preBuild = ''
    export RUSTFLAGS="''${RUSTFLAGS:-} --remap-path-prefix $(rustc --print sysroot)=/rust-toolchain"
  '';
  postInstall = ''
    ln -s $out/bin/termcopy $out/bin/tc
  '';
  meta = with pkgs.lib; {
    description = cargoToml.package.description;
    homepage = cargoToml.package.repository;
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
