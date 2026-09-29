# bingekit-library

The library logic for playing a folder of episodes, shared by mpv-launcher and
a VR player for the Steam Frame.

`scan` finds the video files in a folder, optionally in its subfolders.
`MediaFile` and `Track` describe a file's audio and subtitle tracks, and each
track's language comes from its title when the title names one, because fan
encodes often carry a wrong tag. `choose_tracks` then picks tracks for every
file from a ranked list of language pairings, such as Japanese audio with
English subtitles before English audio alone, and falls back to a file's first
tracks when no pairing fits.

```rust
use bingekit_library::{MediaFile, Track, TrackKind, choose_tracks, default_pairings};

let file = MediaFile {
    path: "ep01.mkv".into(),
    audio: vec![Track::new(TrackKind::Audio, 1, "und", "English")],
    subtitles: vec![],
};
let choices = choose_tracks(&[file], &default_pairings());
assert_eq!(choices[0].audio, Some(1));
assert_eq!(choices[0].subtitles, None);
```

Reading tracks out of a file is up to the caller. Episode order comes from
[bingekit-episode](https://github.com/benashby/bingekit/tree/main/episode).

Part of [bingekit](https://github.com/benashby/bingekit). Licensed under
[0BSD](LICENSE).
