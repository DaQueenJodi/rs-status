{pkgs ? import <nixpkgs> {}}:
pkgs.mkShell {
  nativeBuildInputs = with pkgs; [rustfmt rust-analyzer rustc cargo xorg.libX11 xorg.libXcursor xorg.libxcb pkg-config];
  RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
}
