//! Picking tracks for every file in a folder from a ranked list of pairings.

use crate::priority::Pairing;
use crate::tracks::MediaFile;

/// The tracks picked for one file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Choice {
    /// The file's path, as in its [`MediaFile`].
    pub path: std::path::PathBuf,
    /// The audio track number, or `None` when the file has no audio.
    pub audio: Option<usize>,
    /// The subtitle track number, or `None` for no subtitles.
    pub subtitles: Option<usize>,
    /// Which pairing matched, as an index into the list passed to
    /// [`choose_tracks`]. `None` means no pairing fit and the file got its first
    /// tracks instead.
    pub pairing: Option<usize>,
}

/// Picks tracks for each file: the first pairing in `pairings` that the file
/// fits wins. A file that fits none gets its first audio track and its first
/// subtitle track, if it has them.
#[must_use]
pub fn choose_tracks(files: &[MediaFile], pairings: &[Pairing]) -> Vec<Choice> {
    files
        .iter()
        .map(|file| {
            for (i, pairing) in pairings.iter().enumerate() {
                if let Some((audio, subtitles)) = pairing.tracks_in(file) {
                    return Choice {
                        path: file.path.clone(),
                        audio: Some(audio),
                        subtitles,
                        pairing: Some(i),
                    };
                }
            }
            Choice {
                path: file.path.clone(),
                audio: file.audio.first().map(|t| t.number),
                subtitles: file.subtitles.first().map(|t| t.number),
                pairing: None,
            }
        })
        .collect()
}

/// How many files each pairing covered, in pairing order, with `Fallback` last
/// for files that fit none. Pairings that matched nothing are left out.
#[must_use]
pub fn summary(choices: &[Choice], pairings: &[Pairing]) -> Vec<(String, usize)> {
    let mut counts = vec![0; pairings.len()];
    let mut fallback = 0;
    for choice in choices {
        match choice.pairing {
            Some(i) if i < counts.len() => counts[i] += 1,
            _ => fallback += 1,
        }
    }
    let mut rows: Vec<(String, usize)> = pairings
        .iter()
        .zip(counts)
        .filter(|(_, n)| *n > 0)
        .map(|(p, n)| (p.label.clone(), n))
        .collect();
    if fallback > 0 {
        rows.push(("Fallback".to_owned(), fallback));
    }
    rows
}
