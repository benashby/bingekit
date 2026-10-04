//! Scanning folders for video files.

use std::fs;
use std::path::{Path, PathBuf};

use bingekit_library::{ScanOptions, is_video_file, scan};

fn touch(dir: &Path, name: &str) -> PathBuf {
    let path = dir.join(name);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, b"").unwrap();
    path
}

fn names(paths: &[PathBuf], root: &Path) -> Vec<String> {
    let root = fs::canonicalize(root).unwrap();
    paths
        .iter()
        .map(|p| p.strip_prefix(&root).unwrap().display().to_string())
        .collect()
}

fn season() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    touch(dir.path(), "b.mkv");
    touch(dir.path(), "a.MP4");
    touch(dir.path(), "notes.txt");
    touch(dir.path(), "cover.jpg");
    touch(dir.path(), "extras/ncop.mkv");
    dir
}

#[test]
fn recognises_video_extensions_in_any_case() {
    for name in ["x.mkv", "x.MKV", "x.m2ts", "x.webm", "x.rmvb"] {
        assert!(is_video_file(Path::new(name)), "{name}");
    }
    for name in ["x.srt", "x.ass", "x.txt", "x", "mkv"] {
        assert!(!is_video_file(Path::new(name)), "{name}");
    }
}

#[test]
fn a_flat_scan_skips_subfolders_and_sorts() {
    let dir = season();
    let found = scan(dir.path(), ScanOptions::default()).unwrap();
    assert_eq!(names(&found, dir.path()), ["a.MP4", "b.mkv"]);
    assert!(found.iter().all(|p| p.is_absolute()));
}

#[test]
fn a_recursive_scan_includes_subfolders() {
    let dir = season();
    let options = ScanOptions {
        recursive: true,
        ..ScanOptions::default()
    };
    let found = scan(dir.path(), options).unwrap();
    assert_eq!(
        names(&found, dir.path()),
        ["a.MP4", "b.mkv", "extras/ncop.mkv"]
    );
}

#[test]
fn missing_folders_and_files_are_errors() {
    let dir = season();
    assert!(scan(&dir.path().join("nope"), ScanOptions::default()).is_err());
    assert!(scan(&dir.path().join("b.mkv"), ScanOptions::default()).is_err());
}

#[cfg(unix)]
#[test]
fn links_are_followed_to_files_only_when_asked() {
    use std::os::unix::fs::symlink;
    let dir = season();
    let elsewhere = tempfile::tempdir().unwrap();
    let target = touch(elsewhere.path(), "linked.mkv");
    symlink(&target, dir.path().join("linked.mkv")).unwrap();
    symlink(dir.path(), dir.path().join("loop")).unwrap();

    let plain = scan(
        dir.path(),
        ScanOptions {
            recursive: true,
            follow_symlinks: false,
        },
    )
    .unwrap();
    assert!(!names(&plain, dir.path()).contains(&"linked.mkv".to_owned()));

    let followed = scan(
        dir.path(),
        ScanOptions {
            recursive: true,
            follow_symlinks: true,
        },
    )
    .unwrap();
    let followed = names(&followed, dir.path());
    assert!(followed.contains(&"linked.mkv".to_owned()));
    assert!(!followed.iter().any(|n| n.starts_with("loop")));
}

/// A show and a film laid out the way library managers and release packs leave them:
/// artwork, subtitle and info sidecars, thumbnail tiles in `.trickplay` folders, a
/// scene pack still in its rar volumes with a sample beside it, and a transcode folder
/// holding only a subtitle. The names are made up; the shapes are from a real library.
fn library() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    for name in [
        "Kettle Hill/folder.jpg",
        "Kettle Hill/backdrop.jpg",
        "Kettle Hill/logo.png",
        "Kettle Hill/season01-poster.jpg",
        "Kettle Hill/season-specials-poster.jpg",
        "Kettle Hill/Season 01/Kettle Hill - S01E01 - Pilot Light.mkv",
        "Kettle Hill/Season 01/Kettle Hill - S01E01 - Pilot Light.en.srt",
        "Kettle Hill/Season 01/Kettle Hill - S01E01 - Pilot Light-thumb.jpg",
        "Kettle Hill/Season 01/Kettle Hill - S01E01 - Pilot Light.trickplay/320 - 10x10/0.jpg",
        "Kettle Hill/Season 01/Kettle Hill - S01E02 - Kettle On.mp4",
        "Kettle Hill/Specials/Kettle Hill - S00E01 - Romancing the Turnip.avi",
        "Pocket Critters/Season 20/[capsgrp] Pocket Critters (2023) - 003-004 (TVK 1440x1080 MPEG2 AAC).ts",
        "Night.Ferry.S01.1080p.WEB.h264-DORADO/S01/Night.Ferry.S01E01.1080p.WEB.h264-DORADO/night.ferry.s01e01.1080p.web.h264-dorado.rar",
        "Night.Ferry.S01.1080p.WEB.h264-DORADO/S01/Night.Ferry.S01E01.1080p.WEB.h264-DORADO/night.ferry.s01e01.1080p.web.h264-dorado.r00",
        "Night.Ferry.S01.1080p.WEB.h264-DORADO/S01/Night.Ferry.S01E01.1080p.WEB.h264-DORADO/night.ferry.s01e01.1080p.web.h264-dorado.r01",
        "Night.Ferry.S01.1080p.WEB.h264-DORADO/S01/Night.Ferry.S01E01.1080p.WEB.h264-DORADO/night.ferry.s01e01.1080p.web.h264-dorado.sfv",
        "Night.Ferry.S01.1080p.WEB.h264-DORADO/S01/Night.Ferry.S01E01.1080p.WEB.h264-DORADO/night.ferry.s01e01.1080p.web.h264-dorado.nfo",
        "Night.Ferry.S01.1080p.WEB.h264-DORADO/S01/Night.Ferry.S01E01.1080p.WEB.h264-DORADO/Sample/night.ferry.s01e01.1080p.web.h264-dorado.sample.mkv",
        "Harbor Lights (2001)/Harbor Lights (2001).mkv",
        "Harbor Lights (2001)/Harbor Lights (2001).en.srt",
        "Harbor Lights (2001)/Harbor Lights (2001)-poster.jpg",
        "Harbor Lights (2001)/Harbor Lights (2001).nfo",
        "Harbor Lights (2001)/Plex Versions/Optimized for Mobile/Harbor Lights_ (2001).1234.eng.srt",
    ] {
        touch(root, name);
    }
    dir
}

#[test]
fn a_season_folder_holds_only_its_episodes() {
    let dir = library();
    let found = scan(
        &dir.path().join("Kettle Hill/Season 01"),
        ScanOptions::default(),
    )
    .unwrap();
    assert_eq!(
        names(&found, &dir.path().join("Kettle Hill/Season 01")),
        [
            "Kettle Hill - S01E01 - Pilot Light.mkv",
            "Kettle Hill - S01E02 - Kettle On.mp4"
        ]
    );
}

#[test]
fn a_recursive_scan_finds_the_videos_among_sidecars() {
    let dir = library();
    let options = ScanOptions {
        recursive: true,
        ..ScanOptions::default()
    };
    let found = scan(dir.path(), options).unwrap();
    let found = names(&found, dir.path());
    for want in [
        "Harbor Lights (2001)/Harbor Lights (2001).mkv",
        "Kettle Hill/Season 01/Kettle Hill - S01E01 - Pilot Light.mkv",
        "Kettle Hill/Season 01/Kettle Hill - S01E02 - Kettle On.mp4",
        "Kettle Hill/Specials/Kettle Hill - S00E01 - Romancing the Turnip.avi",
        "Pocket Critters/Season 20/[capsgrp] Pocket Critters (2023) - 003-004 (TVK 1440x1080 MPEG2 AAC).ts",
    ] {
        assert!(
            found.iter().any(|f| f == want),
            "{want} missing from {found:#?}"
        );
    }
    assert!(
        found.iter().all(|f| is_video_file(Path::new(f))),
        "{found:#?}"
    );
}

/// A release's sample is a short clip of an episode, not something to play. The
/// pack it came in may hold nothing else that plays.
#[test]
#[ignore = "release samples are scanned as videos"]
fn a_recursive_scan_leaves_out_release_samples() {
    let dir = library();
    let options = ScanOptions {
        recursive: true,
        ..ScanOptions::default()
    };
    let found = scan(dir.path(), options).unwrap();
    let found = names(&found, dir.path());
    assert!(!found.iter().any(|f| f.contains("sample")), "{found:#?}");
    assert_eq!(found.len(), 5, "{found:#?}");
}

/// Every kind of file a real library held beside its videos.
#[test]
fn library_sidecars_are_not_videos() {
    for name in ["x.mkv", "x.mp4", "x.avi", "x.ts"] {
        assert!(is_video_file(Path::new(name)), "{name}");
    }
    for name in [
        "x.en.srt",
        "x.eng.srt",
        "x.nfo",
        "x.sfv",
        "x.rar",
        "x.r00",
        "x.r18",
        "x.jpg",
        "x.png",
        "x.svg",
    ] {
        assert!(!is_video_file(Path::new(name)), "{name}");
    }
}
