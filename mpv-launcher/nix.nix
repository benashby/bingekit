# The mpv-launcher package and its checks. The root flake imports this file.
{
  perSystem =
    {
      craneLib,
      lib,
      pkgs,
      ...
    }:
    let
      # mpv-launcher depends on episode and library by path, so its source
      # holds all three directories and the build runs from this one.
      src = lib.fileset.toSource {
        root = ../.; # not ./. : the path dependencies sit beside this directory
        fileset = lib.fileset.unions [
          (craneLib.fileset.commonCargoSources ./.)
          ./clippy.toml
          (craneLib.fileset.commonCargoSources ../episode)
          ../episode/README.md
          (craneLib.fileset.commonCargoSources ../library)
          ../library/README.md
        ];
      };

      # Probing needs GStreamer's base and good plugins at run time.
      gstPlugins = lib.makeSearchPathOutput "lib" "lib/gstreamer-1.0" (
        with pkgs.gst_all_1;
        [
          gstreamer
          gst-plugins-base
          gst-plugins-good
        ]
      );

      common = {
        inherit src;
        pname = "mpv-launcher";
        strictDeps = true;
        cargoToml = ./Cargo.toml;
        cargoLock = ./Cargo.lock;
        postUnpack = ''
          cd $sourceRoot/mpv-launcher
          sourceRoot=.
        '';
        cargoExtraArgs = "--locked";
        nativeBuildInputs = [ pkgs.pkg-config ];
        buildInputs = with pkgs.gst_all_1; [
          gstreamer
          gst-plugins-base
        ];
      };
      cargoArtifacts = craneLib.buildDepsOnly common;

      # mpv comes from the user's PATH, so their own build and config apply.
      mpv-launcher = craneLib.buildPackage (
        common
        // {
          inherit cargoArtifacts;
          doCheck = false;
          nativeBuildInputs = common.nativeBuildInputs ++ [ pkgs.makeWrapper ];
          postInstall = ''
            wrapProgram $out/bin/mpv-launcher \
              --prefix GST_PLUGIN_SYSTEM_PATH_1_0 : ${gstPlugins}
          '';
          meta.mainProgram = "mpv-launcher";
        }
      );
    in
    {
      packages.mpv-launcher = mpv-launcher;

      checks = {
        inherit mpv-launcher;
        mpv-launcher-test = craneLib.cargoTest (
          common
          // {
            inherit cargoArtifacts;
            nativeCheckInputs = [ pkgs.ffmpeg ];
            GST_PLUGIN_SYSTEM_PATH_1_0 = gstPlugins;
          }
        );
        mpv-launcher-clippy = craneLib.cargoClippy (
          common
          // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- -D warnings";
          }
        );
        mpv-launcher-fmt = craneLib.cargoFmt {
          inherit (common)
            src
            pname
            cargoToml
            postUnpack
            ;
        };
      };
    };
}
