# bingekit-episode

Episode information from media file names, and a viewing order for a folder of them.

General release names such as `Show.S01E03.720p.BluRay.x264-GROUP.mkv` are parsed by
[hunch](https://crates.io/crates/hunch), a Rust port of guessit. Anime fansub releases
such as `[Group] Title - 05v2 (1080p) [CRC].mkv` follow their own conventions, and this
crate handles those itself. `S01` is special 1, `C01`, `NCOP` and `NCED` are creditless
extras, `v2` is a re-release, and resolution tags like `1920x1080` are never read as a
season and episode.

```rust
use bingekit_episode::{Kind, parse_folder};

let names = [
    "[Grp] Show - 02 [1080p].mkv",
    "[Grp] Show - S01 [1080p].mkv",
    "[Grp] Show - 01 [1080p].mkv",
];
let order: Vec<_> = parse_folder(&names)
    .iter()
    .map(|ep| (ep.kind, ep.episode()))
    .collect();
assert_eq!(
    order,
    [(Kind::Regular, Some(1)), (Kind::Regular, Some(2)), (Kind::Special, Some(1))]
);
```

`parse_folder` gives each file the rest of the folder as context, which settles
ambiguous titles, and returns the files in viewing order: episodes, then specials, then
extras.

Part of [bingekit](https://github.com/benashby/bingekit). Licensed under
[0BSD](LICENSE).
