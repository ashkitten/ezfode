{ pkgs ? import <nixpkgs> {} }:

with pkgs; stdenv.mkDerivation {
  name = "env";
  nativeBuildInputs = [
    rustup
    gcc-arm-embedded
  ];
}
