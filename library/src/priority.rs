//! Audio and subtitle language pairings, tried in the order the user ranks them.

use crate::tracks::{MediaFile, TrackKind};

/// An audio language, with or without a subtitle language.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pairing {
    /// A short stable name, such as `jpn_eng`.
    pub id: String,
    /// The audio language, a three-letter code.
    pub audio: String,
    /// The subtitle language, or `None` for no dialogue subtitles. Without
    /// them, a signs-and-songs track in the audio language still turns on.
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

    /// The track numbers this pairing picks in `file`: the audio track, and a
    /// subtitle track or `None`. A pairing with a subtitle language takes the
    /// first dialogue track in it and never a signs-and-songs track. A pairing
    /// without one takes the signs-and-songs track in the audio language, if
    /// the file has one, so English audio still translates on-screen text.
    /// Returns `None` when the file lacks the audio language, or lacks dialogue
    /// subtitles in the language the pairing asks for.
    #[must_use]
    pub fn tracks_in(&self, file: &MediaFile) -> Option<(usize, Option<usize>)> {
        let audio = file.find(TrackKind::Audio, &self.audio)?.number;
        let subtitles = match &self.subtitles {
            None => file.find_signs(&self.audio),
            Some(lang) => Some(file.find_dialogue(lang)?),
        };
        Some((audio, subtitles.map(|t| t.number)))
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
