{pkgs}:
let 
  source2viewer = pkgs.callPackage ./source2viewer.nix {};
in
pkgs.mkShell rec {
  name = "deadlocked-dev-shell";

  # Libraries that need to be prelinked
  baseLibs = with pkgs; [
    libx11
    libxcursor
    libxkbcommon
    xorg.libXcursor
    xorg.libXi

    libGL
  ];
  nativeBuildInputs = with pkgs;
    [
      # Compilers
      cargo
      rustc
      scdoc

      # Dependencies
      wayland
      source2viewer

      nodejs_24 # radar

      # Tools
      cargo-audit
      cargo-deny
      pkg-config
      clippy
      rust-analyzer
      rustfmt
      strace
      gdb
    ]
    ++ baseLibs;
  LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath baseLibs}";
}
