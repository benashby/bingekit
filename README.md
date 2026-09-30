# bingekit

Media libraries and tools. Each top-level directory is its own project, with its own
language, version, changelog and releases. A project may build on another one by path,
as mpv-launcher does on episode and library; otherwise they share only this repository.

| Project | Language | What it is | Published as |
|---|---|---|---|
| [episode](episode/) | Rust | Episode information from media file names (general and anime release names), and a viewing order for a folder of them | `bingekit-episode` on crates.io |
| [library](library/) | Rust | Video files in a folder, the language of each audio and subtitle track, and per-file track choice from ranked language pairings | `bingekit-library` on crates.io |
| [mpv-launcher](mpv-launcher/) | Rust | A terminal app that plays a folder of episodes in mpv, in episode order, with audio and subtitle tracks picked for each file | Nix package and GitHub release |

Releases are tagged per project as `<project>/vX.Y.Z`, for example `episode/v0.1.0`.

## Building

Every project builds with its own language's tools. For the Rust projects that means
`cargo` inside the project directory.

With Nix, `nix develop` gives a shell with the toolchains and helpers, and
`nix flake check` builds and tests every project. `just` lists each project's recipes.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). The commit and writing rules there are enforced
in CI.

## License

[0BSD](LICENSE). Use it for anything; no attribution needed. Each project carries its
own copy of the license.
