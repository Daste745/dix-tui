{
  inputs = {
    nixpkgs.url = "https://channels.nixos.org/nixpkgs-unstable/nixexprs.tar.zst";
  };

  outputs =
    { nixpkgs, ... }:
    let
      systems = [
        "aarch64-darwin"
        "x86_64-darwin"
        "x86_64-linux"
      ];
      eachSystem = f: nixpkgs.lib.genAttrs systems (system: f (import nixpkgs { inherit system; }));
    in
    {
      packages = eachSystem (
        pkgs:
        let
          dix-tui = pkgs.callPackage ./package.nix { };
        in
        {
          inherit dix-tui;
          default = dix-tui;
        }
      );

      devShells = eachSystem (pkgs: {
        default = pkgs.mkShell {
          packages = with pkgs; [
            pkg-config
            cargo
            clippy
            rust-analyzer
            rustc
            (rustfmt.override { asNightly = true; })
          ];
          shellHook = ''

          '';
        };
      });
    };
}
