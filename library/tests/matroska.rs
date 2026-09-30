//! Reading real Matroska files. The media is made at test time with ffmpeg,
//! so the `ffmpeg` program must be installed.
#![cfg(feature = "matroska")]

use std::process::Command;

use bingekit_library::matroska::{ReadError, read_tracks};
use bingekit_library::{TrackKind, choose_tracks, default_pairings};

mod common;
use common::make_release;

#[test]
fn reads_tracks_in_file_order_with_languages_and_titles() {
    let dir = tempfile::tempdir().unwrap();
    let file = read_tracks(&make_release(dir.path())).unwrap();

    let audio: Vec<_> = file
        .audio
        .iter()
        .map(|t| (t.number, t.language.as_str(), t.title.as_str()))
        .collect();
    assert_eq!(
        audio,
        [
            (1, "jpn", "Japanese"),
            (2, "eng", "English"),
            (3, "fre", "Commentary")
        ]
    );
    let subtitles: Vec<_> = file
        .subtitles
        .iter()
        .map(|t| (t.number, t.language.as_str(), t.title.as_str()))
        .collect();
    assert_eq!(
        subtitles,
        [(1, "eng", "Full Subtitles"), (2, "eng", "Signs & Songs")]
    );
    assert!(file.subtitles.iter().all(|t| t.kind == TrackKind::Subtitle));
}

#[test]
fn a_read_file_goes_straight_into_track_choice() {
    let dir = tempfile::tempdir().unwrap();
    let file = read_tracks(&make_release(dir.path())).unwrap();
    let choice = &choose_tracks(&[file], &default_pairings())[0];
    assert_eq!(
        (choice.pairing, choice.audio, choice.subtitles),
        (Some(0), Some(1), Some(1))
    );
}

#[test]
fn missing_and_other_formats_are_errors() {
    let dir = tempfile::tempdir().unwrap();
    assert!(matches!(
        read_tracks(&dir.path().join("missing.mkv")),
        Err(ReadError::Open(..))
    ));

    let mp4 = dir.path().join("clip.mp4");
    let status = Command::new("ffmpeg")
        .args(["-nostdin", "-loglevel", "error", "-y"])
        .args(["-f", "lavfi", "-i", "sine=frequency=440:duration=1"])
        .arg(&mp4)
        .status()
        .expect("these tests need ffmpeg on PATH");
    assert!(status.success());
    assert!(matches!(read_tracks(&mp4), Err(ReadError::Parse(..))));
}
