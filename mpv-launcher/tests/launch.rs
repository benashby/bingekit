//! Runs the real binary on a folder of generated episodes, with a stand-in
//! `mpv` on `PATH` that records its arguments. The episodes are made with
//! ffmpeg.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Command, Output, Stdio};

/// A one-second Matroska episode with Japanese and English audio and an
/// English subtitle track.
fn make_episode(dir: &Path, name: &str) {
    let subs = dir.join("subs.srt");
    fs::write(&subs, "1\n00:00:00,000 --> 00:00:00,900\nHello\n").unwrap();
    let status = Command::new("ffmpeg")
        .args(["-nostdin", "-loglevel", "error", "-y"])
        .args(["-f", "lavfi", "-i", "testsrc=size=64x64:rate=5:duration=1"])
        .args(["-f", "lavfi", "-i", "sine=frequency=440:duration=1"])
        .args(["-f", "lavfi", "-i", "sine=frequency=660:duration=1"])
        .arg("-i")
        .arg(&subs)
        .args(["-map", "0", "-map", "1", "-map", "2", "-map", "3"])
        .args(["-c:v", "ffv1", "-c:a", "flac", "-c:s", "srt"])
        .args(["-metadata:s:a:0", "language=jpn"])
        .args(["-metadata:s:a:1", "language=eng"])
        .args(["-metadata:s:s:0", "language=eng"])
        .arg(dir.join(name))
        .status()
        .expect("this test needs ffmpeg on PATH");
    assert!(status.success());
    fs::remove_file(subs).unwrap();
}

/// A stand-in for mpv that writes each argument on its own line to
/// `$MPV_ARGS_FILE` and exits with `$MPV_EXIT`.
fn fake_mpv(bin: &Path) {
    let script = bin.join("mpv");
    fs::write(
        &script,
        "#!/bin/sh\nprintf '%s\\n' \"$@\" > \"$MPV_ARGS_FILE\"\nexit \"${MPV_EXIT:-0}\"\n",
    )
    .unwrap();
    fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).unwrap();
}

struct Run {
    output: Output,
    mpv_args: Vec<String>,
}

fn launch(folder: &Path, extra: &[&str], mpv_exit: &str) -> Run {
    let bin = tempfile::tempdir().unwrap();
    fake_mpv(bin.path());
    let args_file = bin.path().join("args");
    let path = format!(
        "{}:{}",
        bin.path().display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let output = Command::new(env!("CARGO_BIN_EXE_mpv-launcher"))
        .arg("-dir")
        .arg(folder)
        .args(extra)
        .env("PATH", path)
        .env("MPV_ARGS_FILE", &args_file)
        .env("MPV_EXIT", mpv_exit)
        .env("XDG_RUNTIME_DIR", bin.path())
        .env_remove("HYPRLAND_INSTANCE_SIGNATURE")
        .stdin(Stdio::null())
        .output()
        .unwrap();
    let mpv_args = fs::read_to_string(&args_file)
        .map(|s| s.lines().map(str::to_owned).collect())
        .unwrap_or_default();
    Run { output, mpv_args }
}

fn season() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    for name in [
        "[Grp] Tidewatch - 02 [1080p].mkv",
        "[Grp] Tidewatch - S01 [1080p].mkv",
        "[Grp] Tidewatch - 01 [1080p].mkv",
    ] {
        make_episode(dir.path(), name);
    }
    dir
}

fn file_names(args: &[String]) -> Vec<String> {
    args.iter()
        .filter(|a| Path::new(a).extension().is_some_and(|e| e == "mkv"))
        .map(|a| {
            Path::new(a)
                .file_name()
                .unwrap()
                .to_string_lossy()
                .into_owned()
        })
        .collect()
}

#[test]
fn plays_the_season_in_order_with_its_tracks() {
    let dir = season();
    let run = launch(dir.path(), &["-profile", "anime"], "0");
    assert!(
        run.output.status.success(),
        "{}",
        String::from_utf8_lossy(&run.output.stderr)
    );
    assert_eq!(
        file_names(&run.mpv_args),
        [
            "[Grp] Tidewatch - 01 [1080p].mkv",
            "[Grp] Tidewatch - 02 [1080p].mkv",
            "[Grp] Tidewatch - S01 [1080p].mkv"
        ]
    );
    // Japanese audio with English subtitles is the first default pairing.
    let first_block: Vec<_> = run
        .mpv_args
        .iter()
        .skip_while(|a| *a != "--{")
        .take(5)
        .collect();
    assert_eq!(
        first_block[..3],
        ["--{", "--aid=1", "--sid=1"],
        "{:?}",
        run.mpv_args
    );
    assert!(run.mpv_args.contains(&"--profile=anime".to_owned()));
    assert!(
        run.mpv_args
            .iter()
            .any(|a| a.starts_with("--input-ipc-server=") && a.ends_with("mpv-launcher.sock"))
    );
    let stdout = String::from_utf8_lossy(&run.output.stdout);
    let stderr = String::from_utf8_lossy(&run.output.stderr);
    for expected in [
        "Found 3 video files",
        "Japanese + Eng Subtitles: 3 files",
        "Playback finished",
    ] {
        assert!(
            stdout.contains(expected),
            "{expected:?} missing from stdout:\n{stdout}\nstderr:\n{stderr}"
        );
    }
}

#[test]
fn an_mpv_failure_is_reported() {
    let dir = season();
    let run = launch(dir.path(), &[], "3");
    assert!(!run.output.status.success());
    assert!(String::from_utf8_lossy(&run.output.stderr).contains("mpv exited"));
}

#[test]
fn an_empty_folder_stops_before_mpv() {
    let dir = tempfile::tempdir().unwrap();
    let run = launch(dir.path(), &[], "0");
    assert!(!run.output.status.success());
    assert!(run.mpv_args.is_empty());
    assert!(String::from_utf8_lossy(&run.output.stdout).contains("No video files found"));
}

#[test]
fn a_file_that_isnt_matroska_plays_in_its_place_with_mpvs_tracks() {
    let dir = season();
    let status = Command::new("ffmpeg")
        .args(["-nostdin", "-loglevel", "error", "-y"])
        .args(["-f", "lavfi", "-i", "testsrc=size=64x64:rate=5:duration=1"])
        .args(["-f", "lavfi", "-i", "sine=frequency=440:duration=1"])
        .args(["-c:v", "mpeg4", "-c:a", "aac"])
        .arg(dir.path().join("[Grp] Tidewatch - 03 [1080p].mp4"))
        .status()
        .expect("this test needs ffmpeg on PATH");
    assert!(status.success());

    let run = launch(dir.path(), &[], "0");
    let stdout = String::from_utf8_lossy(&run.output.stdout);
    assert!(run.output.status.success(), "{stdout}");
    let blocks: Vec<Vec<&str>> = run
        .mpv_args
        .split(|a| a == "--}")
        .map(|b| {
            b.iter()
                .skip_while(|a| *a != "--{")
                .skip(1)
                .map(String::as_str)
                .collect()
        })
        .filter(|b: &Vec<&str>| !b.is_empty())
        .collect();
    let mp4 = &blocks[2];
    assert_eq!(mp4.len(), 1, "{blocks:?}");
    assert!(mp4[0].ends_with("Tidewatch - 03 [1080p].mp4"), "{blocks:?}");
    assert!(
        stdout.contains("Japanese + Eng Subtitles: 3 files"),
        "{stdout}"
    );
    assert!(stdout.contains("Tracks left to mpv: 1 files"), "{stdout}");
}
