{
  lib,
  pkg-config,
  rustPlatform,
  sqlite,
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
  buildInputs = [ sqlite ];

  strictDeps = true;

  meta = {
    description = "TUI for diffing nix systems";
    homepage = "https://github.com/daste745/dix-tui";
    license = lib.licenses.mit;
    mainProgram = "dix-tui";
    maintainers = [ ];
  };
}
