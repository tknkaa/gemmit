{ pkgs }:
pkgs.mkShell {
  # Add build dependencies
  packages = with pkgs; [
    cowsay
    cobra-cli
  ];

  # Add environment variables
  env = { };

  # Load custom bash code
  shellHook = ''

  '';
}
