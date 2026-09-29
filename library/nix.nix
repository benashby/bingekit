# Checks for bingekit-library. The root flake imports this file.
{
  perSystem =
    { craneLib, lib, ... }:
    let
      src = lib.fileset.toSource {
        root = ./.;
        fileset = lib.fileset.unions [
          (craneLib.fileset.commonCargoSources ./.)
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
        library-test = craneLib.cargoTest (common // { inherit cargoArtifacts; });
        library-clippy = craneLib.cargoClippy (
          common
          // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- -D warnings";
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
