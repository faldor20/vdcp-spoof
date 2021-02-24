
let
  sources = import ./nix/sources.nix;
  pkgs = import sources.nixpkgs {};
  rust = import ./nix/rust.nix { inherit sources; };

  

  
  # Get the latest stable compiler version and install the armv7 android target with it
in
  # Note the call to `pkgs` here, instead of `native-pkgs`
  
    # mkShell will drop us into a shell that has a $CC for the target platform
    pkgs.mkShell {
     
      nativeBuildInputs = [rust pkgs.pkg-config ];
      buildInputs=[ pkgs.pkg-config pkgs.udev    ];
    }
   