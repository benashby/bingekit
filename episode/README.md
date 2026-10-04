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

A folder often holds the same episode more than once: two releases side by side, a 720p
copy beside a 1080p one, or a `v2` beside the file it fixed. `play_order` keeps one file
per episode for playback, preferring the release playback started in, and `next_after`
gives the file that plays after the current one. An episode that only another release
has stays in, so a season whose episodes came from different groups still plays through.

```rust
use bingekit_episode::next_after;

let folder = [
    "[GrpA] Show - 01 [1080p].mkv",
    "[GrpB] Show - 01 [720p].mkv",
    "[GrpA] Show - 02 [1080p].mkv",
    "[GrpB] Show - 02 [720p].mkv",
];
let next = next_after(&folder, "[GrpB] Show - 01 [720p].mkv").unwrap();
assert_eq!(next.file_name, "[GrpB] Show - 02 [720p].mkv");
```

Part of [bingekit](https://github.com/benashby/bingekit). Licensed under
[0BSD](LICENSE).
