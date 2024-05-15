let
  pkgs = import <nixpkgs> { };
  packages = with pkgs; [ pkg-config cargo-audit ];
in pkgs.mkShell {
  buildInputs = packages;
  LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath packages}";
}
