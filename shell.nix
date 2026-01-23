{
  pkgs ? import <nixpkgs> { },
}:

with pkgs;
mkShell {
  shellHook = '''';

  nativeBuildInputs = [
    pkg-config
    rustPlatform.bindgenHook
  ];

  buildInputs = [
  ];
}
