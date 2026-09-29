//! Audio and subtitle tracks, and the language each one is really in.

use std::path::PathBuf;

/// Whether a track is audio or subtitles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TrackKind {
    /// An audio track.
    Audio,
    /// A subtitle track.
    Subtitle,
}

/// One audio or subtitle track in a file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Track {
    /// Audio or subtitles.
    pub kind: TrackKind,
    /// 1-based position among the file's tracks of the same kind. mpv's `--aid`
    /// and `--sid` count the same way.
    pub number: usize,
    /// The language tag from the container, such as `jpn`, `en` or `und`. Empty
    /// when the file has none.
    pub language: String,
    /// The track title, such as `Japanese` or `Full Subtitles`. Empty when the
    /// file has none.
    pub title: String,
}

impl Track {
    /// Builds a track from its kind, number, language tag and title.
    #[must_use]
    pub fn new(kind: TrackKind, number: usize, language: &str, title: &str) -> Self {
        Self {
            kind,
            number,
            language: language.to_owned(),
            title: title.to_owned(),
        }
    }

    /// The language the track is actually in. See [`effective_language`].
    #[must_use]
    pub fn language(&self) -> Option<String> {
        effective_language(&self.language, &self.title)
    }
}

/// The audio and subtitle tracks of one file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaFile {
    /// Where the file is.
    pub path: PathBuf,
    /// Audio tracks, in file order.
    pub audio: Vec<Track>,
    /// Subtitle tracks, in file order.
    pub subtitles: Vec<Track>,
}

impl MediaFile {
    /// The tracks of one kind.
    #[must_use]
    pub fn tracks(&self, kind: TrackKind) -> &[Track] {
        match kind {
            TrackKind::Audio => &self.audio,
            TrackKind::Subtitle => &self.subtitles,
        }
    }

    /// The first track of `kind` in `language` (a three-letter code such as
    /// `jpn`), if the file has one.
    #[must_use]
    pub fn find(&self, kind: TrackKind, language: &str) -> Option<&Track> {
        self.tracks(kind)
            .iter()
            .find(|t| t.language().as_deref() == Some(language))
    }

    /// Whether the file has a track of `kind` in `language`.
    #[must_use]
    pub fn has_language(&self, kind: TrackKind, language: &str) -> bool {
        self.find(kind, language).is_some()
    }
}

/// The language a track is in, as a lower-case three-letter code.
///
/// Fan encodes often carry a wrong or missing language tag and a correct title,
/// so a title naming Japanese or English wins over the tag. Otherwise the tag is
/// used, with the two-letter `ja` and `en` read as `jpn` and `eng`. Returns
/// `None` when neither says anything.
#[must_use]
pub fn effective_language(tag: &str, title: &str) -> Option<String> {
    if let Some(lang) = language_in_title(title) {
        return Some(lang.to_owned());
    }
    let tag = tag.trim().to_ascii_lowercase();
    match tag.as_str() {
        "" => None,
        "ja" => Some("jpn".to_owned()),
        "en" => Some("eng".to_owned()),
        _ => Some(tag),
    }
}

/// Looks for a language named in a track title. Only whole words count, so
/// `Bengali` does not read as English.
fn language_in_title(title: &str) -> Option<&'static str> {
    if title.contains("日本語") {
        return Some("jpn");
    }
    let words = title
        .split(|c: char| !c.is_alphanumeric())
        .map(str::to_ascii_lowercase);
    let mut found = None;
    for word in words {
        match word.as_str() {
            "japanese" | "jpn" => return Some("jpn"),
            "english" | "eng" => found = found.or(Some("eng")),
            _ => {}
        }
    }
    found
}
