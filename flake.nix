{
  description = "Gemini API wrapper for professional-like commit message";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        devShells.default = import ./devshell.nix {
          inherit pkgs;
        };
        formatter = pkgs.nixpkgs-fmt;
        packages.default = pkgs.buildGoModule {
          pname = "gemmit";
          version = "0.1.0";
          src = self;
          vendorHash = "sha256-JiTWrRsvRZh6D5/knTBy8fr+6zGj19FuToxglvuqc/0=";
        };
        apps.default = {
          type = "app";
          program = "${self.packages.${system}.default}/bin/gemmit";
        };
      }

    );
}
