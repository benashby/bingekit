//! Probing real files. The media is made at test time with ffmpeg, so the
//! `ffmpeg` program and GStreamer's base and good plugins must be installed.
#![cfg(feature = "gstreamer")]

use std::time::Duration;

use bingekit_library::probe::{ProbeError, Prober};
use bingekit_library::{TrackKind, choose_tracks, default_pairings};

mod common;
use common::make_release;

fn prober() -> Prober {
    Prober::new(Duration::from_secs(10)).expect("GStreamer and its Discoverer")
}

#[test]
fn reads_tracks_in_file_order_with_tags_and_titles() {
    let dir = tempfile::tempdir().unwrap();
    let file = prober().probe(&make_release(dir.path())).unwrap();

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
    let subtitles: Vec<_> = file.subtitles.iter().map(|t| t.title.as_str()).collect();
    assert_eq!(subtitles, ["Full Subtitles", "Signs & Songs"]);
    assert!(file.subtitles.iter().all(|t| t.kind == TrackKind::Subtitle));
}

#[test]
fn a_probed_file_goes_straight_into_track_choice() {
    let dir = tempfile::tempdir().unwrap();
    let file = prober().probe(&make_release(dir.path())).unwrap();
    let choice = &choose_tracks(&[file], &default_pairings())[0];
    assert_eq!(
        (choice.pairing, choice.audio, choice.subtitles),
        (Some(0), Some(1), Some(1))
    );
}

#[test]
fn missing_and_unreadable_files_are_errors() {
    let dir = tempfile::tempdir().unwrap();
    let prober = prober();
    assert!(matches!(
        prober.probe(&dir.path().join("missing.mkv")),
        Err(ProbeError::Path(_))
    ));
    let junk = dir.path().join("junk.mkv");
    std::fs::write(&junk, b"not a video").unwrap();
    assert!(matches!(prober.probe(&junk), Err(ProbeError::Discover(..))));
}
