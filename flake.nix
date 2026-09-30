{
  description = "bingekit: media libraries and tools, one project per directory";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    crane.url = "github:ipetkov/crane";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    inputs@{ flake-parts, ... }:
    let
      # Every top-level directory with a nix.nix is a project. Each one writes its
      # sources as paths relative to itself, so Nix hashes projects independently.
      entries = builtins.readDir ./.;
      projects = builtins.filter (
        name: entries.${name} == "directory" && builtins.pathExists (./. + "/${name}/nix.nix")
      ) (builtins.attrNames entries);
    in
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];

      imports = map (name: ./. + "/${name}/nix.nix") projects;

      perSystem =
        { pkgs, system, ... }:
        let
          # The development toolchain. Libraries still declare their own rust-version,
          # and CI builds each one on that version too.
          rustToolchain = pkgs.rust-bin.stable."1.98.1".default;
        in
        {
          _module.args = {
            pkgs = import inputs.nixpkgs {
              inherit system;
              overlays = [ inputs.rust-overlay.overlays.default ];
            };
            craneLib = (inputs.crane.mkLib pkgs).overrideToolchain rustToolchain;
          };

          devShells.default = pkgs.mkShell {
            packages = with pkgs; [
              rustToolchain
              just
              cargo-insta
              cargo-deny
              # bingekit-library's gstreamer feature. ffmpeg makes the media
              # that the Matroska, GStreamer and mpv-launcher tests read.
              pkg-config
              gst_all_1.gstreamer
              gst_all_1.gst-plugins-base
              gst_all_1.gst-plugins-good
              ffmpeg
            ];
          };

          formatter = pkgs.nixfmt;
        };
    };
}
