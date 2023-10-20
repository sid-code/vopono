{
  inputs = {
    naersk.url = "github:nix-community/naersk/master";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs = {
    self,
    nixpkgs,
    naersk,
  }: let
    supportedSystems = ["x86_64-linux"];
    forEachSystem = f:
      nixpkgs.lib.genAttrs supportedSystems (system:
        f {
          inherit system;
          pkgs = import nixpkgs {inherit system;};
        });
  in {
    packages = forEachSystem ({
      system,
      pkgs,
    }: {default = (pkgs.callPackage naersk {}).buildPackage ./.;});

    devShells = forEachSystem ({
      system,
      pkgs,
    }: {
      default = with pkgs;
        mkShell {
          buildInputs = [
            cargo
            rustc
            rustfmt
            pre-commit
            rustPackages.clippy
            rust-analyzer
            wireguard-tools
          ];
          RUST_SRC_PATH = rustPlatform.rustLibSrc;
        };
    });
  };
}
