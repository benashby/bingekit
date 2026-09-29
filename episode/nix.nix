# Checks for bingekit-episode. The root flake imports this file.
{
  perSystem =
    { craneLib, lib, ... }:
    let
      src = lib.fileset.toSource {
        root = ./.;
        fileset = lib.fileset.unions [
          (craneLib.fileset.commonCargoSources ./.)
          ./testdata
          ./tests/snapshots
          ./README.md
        ];
      };
      common = {
        inherit src;
        strictDeps = true;
      };
      cargoArtifacts = craneLib.buildDepsOnly common;
    in
    {
      checks = {
        episode-test = craneLib.cargoTest (common // { inherit cargoArtifacts; });
        episode-clippy = craneLib.cargoClippy (
          common
          // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- -D warnings";
          }
        );
        episode-fmt = craneLib.cargoFmt { inherit src; };
        episode-doc = craneLib.cargoDoc (
          common
          // {
            inherit cargoArtifacts;
            RUSTDOCFLAGS = "-Dwarnings";
          }
        );
      };
    };
}
