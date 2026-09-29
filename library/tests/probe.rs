//! Probing real files. The media is made at test time with ffmpeg, so the
//! `ffmpeg` program and GStreamer's base and good plugins must be installed.
#![cfg(feature = "gstreamer")]

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use bingekit_library::probe::{ProbeError, Prober};
use bingekit_library::{TrackKind, choose_tracks, default_pairings};

/// A one-second Matroska file with three audio tracks and two subtitle tracks,
/// tagged the way a dual-audio anime release is.
fn make_release(dir: &Path) -> PathBuf {
    let subs = dir.join("subs.srt");
    std::fs::write(&subs, "1\n00:00:00,000 --> 00:00:00,900\nHello\n").unwrap();
    let out = dir.join("ep01.mkv");
    let status = Command::new("ffmpeg")
        .args(["-nostdin", "-loglevel", "error", "-y"])
        .args(["-f", "lavfi", "-i", "testsrc=size=64x64:rate=5:duration=1"])
        .args(["-f", "lavfi", "-i", "sine=frequency=440:duration=1"])
        .args(["-f", "lavfi", "-i", "sine=frequency=660:duration=1"])
        .args(["-f", "lavfi", "-i", "sine=frequency=880:duration=1"])
        .arg("-i")
        .arg(&subs)
        .arg("-i")
        .arg(&subs)
        .args([
            "-map", "0", "-map", "1", "-map", "2", "-map", "3", "-map", "4", "-map", "5",
        ])
        .args(["-c:v", "ffv1", "-c:a", "flac", "-c:s", "srt"])
        .args([
            "-metadata:s:a:0",
            "language=jpn",
            "-metadata:s:a:0",
            "title=Japanese",
        ])
        .args([
            "-metadata:s:a:1",
            "language=eng",
            "-metadata:s:a:1",
            "title=English",
        ])
        .args([
            "-metadata:s:a:2",
            "language=fre",
            "-metadata:s:a:2",
            "title=Commentary",
        ])
        .args([
            "-metadata:s:s:0",
            "language=eng",
            "-metadata:s:s:0",
            "title=Full Subtitles",
        ])
        .args([
            "-metadata:s:s:1",
            "language=eng",
            "-metadata:s:s:1",
            "title=Signs & Songs",
        ])
        .arg(&out)
        .status()
        .expect("these tests need ffmpeg on PATH");
    assert!(status.success(), "ffmpeg failed to make the test file");
    out
}

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
