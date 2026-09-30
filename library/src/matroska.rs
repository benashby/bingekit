//! Reading a Matroska file's audio and subtitle tracks from its headers.
//!
//! Needs the `matroska` feature. Pure Rust: it reads the track entries at the
//! start of the file and never decodes anything, so the answer is the same on
//! every run. Matroska covers `.mkv`, `.mka` and `.webm`.

use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use matroska_demuxer::{MatroskaFile, TrackEntry, TrackType};

use crate::tracks::{MediaFile, Track, TrackKind};

/// Why a file's tracks could not be read.
#[derive(Debug)]
pub enum ReadError {
    /// The file could not be opened.
    Open(PathBuf, std::io::Error),
    /// The file is not Matroska, or its headers are damaged.
    Parse(PathBuf, matroska_demuxer::DemuxError),
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Open(p, e) => write!(f, "could not open {}: {e}", p.display()),
            Self::Parse(p, e) => write!(f, "could not read {} as Matroska: {e}", p.display()),
        }
    }
}

impl Error for ReadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Open(_, e) => Some(e),
            Self::Parse(_, e) => Some(e),
        }
    }
}

/// Reads the audio and subtitle tracks of the Matroska file at `path`. Tracks
/// are numbered from 1 per kind, in the order the file lists them, which is
/// how mpv numbers them too.
///
/// # Errors
///
/// Fails when the file cannot be opened or is not valid Matroska.
pub fn read_tracks(path: &Path) -> Result<MediaFile, ReadError> {
    let file = File::open(path).map_err(|e| ReadError::Open(path.into(), e))?;
    let mkv =
        MatroskaFile::open(BufReader::new(file)).map_err(|e| ReadError::Parse(path.into(), e))?;
    Ok(media_file(path, mkv.tracks()))
}

fn media_file(path: &Path, entries: &[TrackEntry]) -> MediaFile {
    let mut audio = Vec::new();
    let mut subtitles = Vec::new();
    for entry in entries {
        let (kind, list) = match entry.track_type() {
            TrackType::Audio => (TrackKind::Audio, &mut audio),
            TrackType::Subtitle => (TrackKind::Subtitle, &mut subtitles),
            _ => continue,
        };
        let number = list.len() + 1;
        list.push(Track::new(
            kind,
            number,
            &language(entry.language_bcp47(), entry.language()),
            entry.name().unwrap_or_default(),
        ));
    }
    MediaFile {
        path: path.into(),
        audio,
        subtitles,
    }
}

/// A track's language from its two language elements, as a three-letter
/// ISO 639-2 bibliographic code such as `jpn`, `fre` or `ger`, the form
/// `Language` uses and the rest of this crate expects.
///
/// `LanguageBCP47`, when present, replaces `Language`. Only the primary
/// language is kept, so `ja-JP` becomes `jpn` and `fre-ca` becomes `fre`. With
/// neither element the track is English, which is the default the Matroska
/// spec gives `Language`.
fn language(bcp47: Option<&str>, iso639: Option<&str>) -> String {
    let tag = bcp47.or(iso639).unwrap_or("eng");
    let primary = tag
        .split(['-', '_'])
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    bibliographic(&primary).map_or(primary, str::to_owned)
}

/// The ISO 639-2 bibliographic code for a BCP 47 primary language subtag,
/// when it differs from the subtag.
fn bibliographic(subtag: &str) -> Option<&'static str> {
    match subtag {
        // Mandarin and Cantonese, which ISO 639-2 only knows as Chinese.
        "cmn" | "yue" => Some("chi"),
        _ => ISO_639_1
            .binary_search_by_key(&subtag, |&(two, _)| two)
            .ok()
            .map(|i| ISO_639_1[i].1),
    }
}

/// Every ISO 639-1 code with its ISO 639-2 bibliographic code, sorted.
const ISO_639_1: [(&str, &str); 184] = [
    ("aa", "aar"),
    ("ab", "abk"),
    ("ae", "ave"),
    ("af", "afr"),
    ("ak", "aka"),
    ("am", "amh"),
    ("an", "arg"),
    ("ar", "ara"),
    ("as", "asm"),
    ("av", "ava"),
    ("ay", "aym"),
    ("az", "aze"),
    ("ba", "bak"),
    ("be", "bel"),
    ("bg", "bul"),
    ("bh", "bih"),
    ("bi", "bis"),
    ("bm", "bam"),
    ("bn", "ben"),
    ("bo", "tib"),
    ("br", "bre"),
    ("bs", "bos"),
    ("ca", "cat"),
    ("ce", "che"),
    ("ch", "cha"),
    ("co", "cos"),
    ("cr", "cre"),
    ("cs", "cze"),
    ("cu", "chu"),
    ("cv", "chv"),
    ("cy", "wel"),
    ("da", "dan"),
    ("de", "ger"),
    ("dv", "div"),
    ("dz", "dzo"),
    ("ee", "ewe"),
    ("el", "gre"),
    ("en", "eng"),
    ("eo", "epo"),
    ("es", "spa"),
    ("et", "est"),
    ("eu", "baq"),
    ("fa", "per"),
    ("ff", "ful"),
    ("fi", "fin"),
    ("fj", "fij"),
    ("fo", "fao"),
    ("fr", "fre"),
    ("fy", "fry"),
    ("ga", "gle"),
    ("gd", "gla"),
    ("gl", "glg"),
    ("gn", "grn"),
    ("gu", "guj"),
    ("gv", "glv"),
    ("ha", "hau"),
    ("he", "heb"),
    ("hi", "hin"),
    ("ho", "hmo"),
    ("hr", "hrv"),
    ("ht", "hat"),
    ("hu", "hun"),
    ("hy", "arm"),
    ("hz", "her"),
    ("ia", "ina"),
    ("id", "ind"),
    ("ie", "ile"),
    ("ig", "ibo"),
    ("ii", "iii"),
    ("ik", "ipk"),
    ("io", "ido"),
    ("is", "ice"),
    ("it", "ita"),
    ("iu", "iku"),
    ("ja", "jpn"),
    ("jv", "jav"),
    ("ka", "geo"),
    ("kg", "kon"),
    ("ki", "kik"),
    ("kj", "kua"),
    ("kk", "kaz"),
    ("kl", "kal"),
    ("km", "khm"),
    ("kn", "kan"),
    ("ko", "kor"),
    ("kr", "kau"),
    ("ks", "kas"),
    ("ku", "kur"),
    ("kv", "kom"),
    ("kw", "cor"),
    ("ky", "kir"),
    ("la", "lat"),
    ("lb", "ltz"),
    ("lg", "lug"),
    ("li", "lim"),
    ("ln", "lin"),
    ("lo", "lao"),
    ("lt", "lit"),
    ("lu", "lub"),
    ("lv", "lav"),
    ("mg", "mlg"),
    ("mh", "mah"),
    ("mi", "mao"),
    ("mk", "mac"),
    ("ml", "mal"),
    ("mn", "mon"),
    ("mr", "mar"),
    ("ms", "may"),
    ("mt", "mlt"),
    ("my", "bur"),
    ("na", "nau"),
    ("nb", "nob"),
    ("nd", "nde"),
    ("ne", "nep"),
    ("ng", "ndo"),
    ("nl", "dut"),
    ("nn", "nno"),
    ("no", "nor"),
    ("nr", "nbl"),
    ("nv", "nav"),
    ("ny", "nya"),
    ("oc", "oci"),
    ("oj", "oji"),
    ("om", "orm"),
    ("or", "ori"),
    ("os", "oss"),
    ("pa", "pan"),
    ("pi", "pli"),
    ("pl", "pol"),
    ("ps", "pus"),
    ("pt", "por"),
    ("qu", "que"),
    ("rm", "roh"),
    ("rn", "run"),
    ("ro", "rum"),
    ("ru", "rus"),
    ("rw", "kin"),
    ("sa", "san"),
    ("sc", "srd"),
    ("sd", "snd"),
    ("se", "sme"),
    ("sg", "sag"),
    ("si", "sin"),
    ("sk", "slo"),
    ("sl", "slv"),
    ("sm", "smo"),
    ("sn", "sna"),
    ("so", "som"),
    ("sq", "alb"),
    ("sr", "srp"),
    ("ss", "ssw"),
    ("st", "sot"),
    ("su", "sun"),
    ("sv", "swe"),
    ("sw", "swa"),
    ("ta", "tam"),
    ("te", "tel"),
    ("tg", "tgk"),
    ("th", "tha"),
    ("ti", "tir"),
    ("tk", "tuk"),
    ("tl", "tgl"),
    ("tn", "tsn"),
    ("to", "ton"),
    ("tr", "tur"),
    ("ts", "tso"),
    ("tt", "tat"),
    ("tw", "twi"),
    ("ty", "tah"),
    ("ug", "uig"),
    ("uk", "ukr"),
    ("ur", "urd"),
    ("uz", "uzb"),
    ("ve", "ven"),
    ("vi", "vie"),
    ("vo", "vol"),
    ("wa", "wln"),
    ("wo", "wol"),
    ("xh", "xho"),
    ("yi", "yid"),
    ("yo", "yor"),
    ("za", "zha"),
    ("zh", "chi"),
    ("zu", "zul"),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bcp47_wins_and_only_the_primary_language_is_kept() {
        assert_eq!(language(Some("ja-JP"), Some("eng")), "jpn");
        assert_eq!(language(None, Some("fre-ca")), "fre");
        assert_eq!(language(None, Some("JPN")), "jpn");
        assert_eq!(language(Some("und"), Some("und")), "und");
    }

    #[test]
    fn bcp47_codes_become_bibliographic() {
        assert_eq!(language(Some("fr"), None), "fre");
        assert_eq!(language(Some("de"), None), "ger");
        assert_eq!(language(Some("zh-Hant"), None), "chi");
        assert_eq!(language(Some("cmn"), None), "chi");
        assert_eq!(language(Some("es-419"), None), "spa");
        assert_eq!(language(Some("fil"), None), "fil");
    }

    #[test]
    fn the_table_is_sorted_for_binary_search() {
        assert!(ISO_639_1.windows(2).all(|w| w[0].0 < w[1].0));
        assert!(
            ISO_639_1
                .iter()
                .all(|(two, three)| two.len() == 2 && three.len() == 3)
        );
    }

    #[test]
    fn a_track_without_a_language_is_english() {
        assert_eq!(language(None, None), "eng");
    }
}
