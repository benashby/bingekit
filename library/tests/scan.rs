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
