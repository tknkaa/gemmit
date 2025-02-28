{
  description = "Gemmit";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    nixpkgs,
    flake-utils,
    rust-overlay,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      overlays = [(import rust-overlay)];
      pkgs = import nixpkgs {
        inherit system overlays;
      };
      version = (fromTOML (builtins.readFile ./Cargo.toml)).package.version;
      toolchain = pkgs.rust-bin.stable.latest.default;
      package = pkgs.callPackage ./default.nix {inherit version toolchain;};
    in {
      devShells.default = pkgs.mkShell {
        packages = [toolchain];
      };
      packages.gemmit = package;
      packages.default = package;
    });
}
