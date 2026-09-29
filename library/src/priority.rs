//! Audio and subtitle language pairings, tried in the order the user ranks them.

use crate::tracks::{MediaFile, TrackKind};

/// An audio language, with or without a subtitle language.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pairing {
    /// A short stable name, such as `jpn_eng`.
    pub id: String,
    /// The audio language, a three-letter code.
    pub audio: String,
    /// The subtitle language, or `None` for no subtitles.
    pub subtitles: Option<String>,
    /// What the user sees, such as `Japanese + Eng Subtitles`.
    pub label: String,
}

impl Pairing {
    /// Builds a pairing.
    #[must_use]
    pub fn new(id: &str, audio: &str, subtitles: Option<&str>, label: &str) -> Self {
        Self {
            id: id.to_owned(),
            audio: audio.to_owned(),
            subtitles: subtitles.map(str::to_owned),
            label: label.to_owned(),
        }
    }

    /// The track numbers this pairing picks in `file`: the audio track, and the
    /// subtitle track or `None` when subtitles are off. Returns `None` when the
    /// file lacks the audio language, or lacks the subtitle language the
    /// pairing asks for.
    #[must_use]
    pub fn tracks_in(&self, file: &MediaFile) -> Option<(usize, Option<usize>)> {
        let audio = file.find(TrackKind::Audio, &self.audio)?.number;
        match &self.subtitles {
            None => Some((audio, None)),
            Some(lang) => {
                let subtitles = file.find(TrackKind::Subtitle, lang)?.number;
                Some((audio, Some(subtitles)))
            }
        }
    }

    /// Whether `file` has the tracks this pairing needs.
    #[must_use]
    pub fn fits(&self, file: &MediaFile) -> bool {
        self.tracks_in(file).is_some()
    }
}

/// The four pairings mpv-launcher offers, in its default order.
#[must_use]
pub fn default_pairings() -> Vec<Pairing> {
    vec![
        Pairing::new("jpn_eng", "jpn", Some("eng"), "Japanese + Eng Subtitles"),
        Pairing::new("eng_none", "eng", None, "English - No Subtitles"),
        Pairing::new("jpn_none", "jpn", None, "Japanese - No Subtitles"),
        Pairing::new("eng_eng", "eng", Some("eng"), "English + Eng Subtitles"),
    ]
}
