# Gemmit

set your api key
```sh
export GEMINI_API_KEY=your_api_key
```

build and publish
```sh
cargo build --release
cp target/release/gemmit bin/
npm login
cd bin
npm version patch # update patch version
npm version minor # update minor version
npm version major # update major version
npm publish --access public
```

## Installation

Run directly (automatically builds if needed)

```sh
nix run github:tknkaa/gemmit
```

Via Nix Flakes

```nix
{
  inputs = {
    # other inputs...
    gemmit.url = "github:tknkaa/gemmit";
    gemmit.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = inputs: let
    system = "x86_64-linux";
    pkgs = inputs.nixpkgs.legacyPackages.${system};
  in {
    devShells.${system}.default = pkgs.mkShell {
      packages = [
        # use `inputs.gemmit.packages.${system}.default` to install gemmit
        inputs.gemmit.packages.${system}.default
      ];
    };
  };
}
```
