# Gemmit

set your api key
```
export GEMINI_API_KEY=your_api_key
```

build and publish
```
cargo build --release
cp target/release/gemmit pkg/
npm login
cd pkg
npm version patch # パッチバージョンを更新
npm version minor # マイナーバージョンを更新
npm version major # メジャーバージョンを更新
npm publish --access public
```

## Installation

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
