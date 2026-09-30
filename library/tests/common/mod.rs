//! Media shared by the integration tests, made at test time with ffmpeg.
#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::process::Command;

/// A one-second Matroska file with three audio tracks and two subtitle tracks,
/// tagged the way a dual-audio anime release is.
pub fn make_release(dir: &Path) -> PathBuf {
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
