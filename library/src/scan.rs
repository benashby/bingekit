//! Finding the video files in a folder.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Video file extensions, in lower case and without the dot.
const VIDEO_EXTENSIONS: &[&str] = &[
    "mp4", "mkv", "avi", "mov", "wmv", "flv", "webm", "m4v", "mpg", "mpeg", "m2v", "3gp", "3g2",
    "ogv", "ts", "mts", "m2ts", "vob", "asf", "rm", "rmvb", "divx", "dv", "f4v", "mxf", "roq",
    "yuv", "nsv", "gxf", "qt", "xvid", "mp2", "mpe", "mpv", "m4p", "m4b", "dat", "vcd", "svcd",
    "drc", "gif", "gifv", "mng", "viv", "amv", "m2p", "m2t", "m4s", "tod", "vro", "wtv",
];

/// How [`scan`] walks a folder.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ScanOptions {
    /// Look inside subfolders too.
    pub recursive: bool,
    /// Include files reached through symbolic links. Linked folders are never
    /// entered, which rules out cycles.
    pub follow_symlinks: bool,
}

/// Whether a file name has a video extension. Case does not matter.
#[must_use]
pub fn is_video_file(name: &Path) -> bool {
    name.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| VIDEO_EXTENSIONS.contains(&e.to_ascii_lowercase().as_str()))
}

/// The video files in `dir`, as absolute paths in sorted order.
///
/// # Errors
///
/// Fails when `dir` does not exist, is not a folder, or cannot be read.
/// Subfolders that cannot be read during a recursive scan are skipped.
pub fn scan(dir: &Path, options: ScanOptions) -> io::Result<Vec<PathBuf>> {
    let dir = fs::canonicalize(dir)?;
    if !dir.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::NotADirectory,
            format!("{} is not a directory", dir.display()),
        ));
    }
    let mut found = Vec::new();
    walk(&dir, options, true, &mut found)?;
    found.sort();
    Ok(found)
}

fn walk(dir: &Path, options: ScanOptions, top: bool, found: &mut Vec<PathBuf>) -> io::Result<()> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) if top => return Err(e),
        Err(_) => return Ok(()),
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(kind) = entry.file_type() else {
            continue;
        };
        if kind.is_symlink() {
            // A link is followed only to a file. A link to a folder is skipped
            // either way, so the walk cannot loop.
            if options.follow_symlinks && path.is_file() && is_video_file(&path) {
                found.push(path);
            }
        } else if kind.is_dir() {
            if options.recursive {
                walk(&path, options, false, found)?;
            }
        } else if is_video_file(&path) {
            found.push(path);
        }
    }
    Ok(())
}
