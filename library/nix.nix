# Checks for bingekit-library. The root flake imports this file.
{
  perSystem =
    {
      craneLib,
      lib,
      pkgs,
      ...
    }:
    let
      src = lib.fileset.toSource {
        root = ./.;
        fileset = lib.fileset.unions [
          (craneLib.fileset.commonCargoSources ./.)
          ./README.md
          ./clippy.toml
        ];
      };
      common = {
        inherit src;
        strictDeps = true;
      };
      cargoArtifacts = craneLib.buildDepsOnly common;

      # The gstreamer feature links GStreamer, and its tests make media with
      # ffmpeg and read it back with the good plugins.
      withGstreamer = common // {
        cargoExtraArgs = "--locked --features gstreamer";
        nativeBuildInputs = [ pkgs.pkg-config ];
        buildInputs = with pkgs.gst_all_1; [
          gstreamer
          gst-plugins-base
        ];
      };
      gstreamerArtifacts = craneLib.buildDepsOnly withGstreamer;
    in
    {
      checks = {
        library-test = craneLib.cargoTest (common // { inherit cargoArtifacts; });
        library-clippy = craneLib.cargoClippy (
          common
          // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- -D warnings";
          }
        );
        library-gstreamer-test = craneLib.cargoTest (
          withGstreamer
          // {
            cargoArtifacts = gstreamerArtifacts;
            nativeCheckInputs = [ pkgs.ffmpeg ];
            GST_PLUGIN_SYSTEM_PATH_1_0 = lib.makeSearchPathOutput "lib" "lib/gstreamer-1.0" (
              with pkgs.gst_all_1;
              [
                gstreamer
                gst-plugins-base
                gst-plugins-good
              ]
            );
          }
        );
        library-fmt = craneLib.cargoFmt { inherit src; };
        library-doc = craneLib.cargoDoc (
          common
          // {
            inherit cargoArtifacts;
            RUSTDOCFLAGS = "-Dwarnings";
          }
        );
      };
    };
}
