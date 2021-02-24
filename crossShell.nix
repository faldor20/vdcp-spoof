
let
  sources = import ./nix/sources.nix;
  pkgs = import sources.nixpkgs {};
  rust = import ./nix/rust_win.nix { inherit sources; };

  
  crPkgs = import sources.nixpkgs {
    crossSystem = {config = "x86_64-w64-mingw32";};
  };
  
  # Get the latest stable compiler version and install the armv7 android target with it
in
  # Note the call to `pkgs` here, instead of `native-pkgs`
  
    # mkShell will drop us into a shell that has a $CC for the target platform
    crPkgs.mkShell {
    # removed without issue: crPkgs.windows.mingw_w64_pthreads crPkgs.windows.mingw_w64
    #mingw_w64 must be in nativeBuildInputs
      
      nativeBuildInputs=[   /*crPkgs.windows.mingw_w64*/ rust  ];
     # i used to have windows.pthreads instea of mingw_w64_pthreads. I get errors at the beginning using pthreads then i get errors at the end if i don't
     #crPkgs.windows.pthreads must be in buildinputs
      buildInputs=[        /* crPkgs.windows.pthreads */ crPkgs.windows.mingw_w64_pthreads      ];
      
    }
    