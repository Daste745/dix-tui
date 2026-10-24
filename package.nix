{
  pkg-config,
  rustPlatform,
}:
rustPlatform.buildRustPackage {
  name = "dix-tui";

  src = ./.;

  cargoLock = {
    lockFile = ./Cargo.lock;
    allowBuiltinFetchGit = true;
  };

  buildType = "release";

  nativeBuildInputs = [ pkg-config ];

  strictDeps = true;
}
