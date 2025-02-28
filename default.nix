{
  makeRustPlatform,
  openssl,
  pkg-config,
  toolchain,
  version,
}: let
  rustPlatform = makeRustPlatform {
    cargo = toolchain;
    rustc = toolchain;
  };
in
  rustPlatform.buildRustPackage {
    nativeBuildInputs = [pkg-config];
    buildInputs = [openssl];
    inherit version;
    pname = "gemmit";
    src = ./.;
    cargoLock.lockFile = ./Cargo.lock;
    env = {
      PKG_CONFIG_PATH = "${openssl.dev}/lib/pkgconfig";
    };
  }
