let 
  nixpkgs = import <nixpkgs> {};
in
  with nixpkgs;
  stdenv.mkDerivation {
    name = "brightctl";
    buildInputs = [ 
      gcc
    ];
  }
