{
  lib,
  pkg-config,
  rustPlatform,
  rev ? "dirty",
}:
let
  cargoToml = lib.importTOML ./Cargo.toml;
in
rustPlatform.buildRustPackage {
  pname = "dix-tui";
  version = "${cargoToml.package.version}-${rev}";

  src = ./.;

  cargoLock = {
    lockFile = ./Cargo.lock;
    allowBuiltinFetchGit = true;
  };

  buildType = "release";

  nativeBuildInputs = [ pkg-config ];

  strictDeps = true;
}
