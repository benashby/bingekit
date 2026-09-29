//! Reading a file's audio and subtitle tracks with GStreamer's Discoverer.
//!
//! Needs the `gstreamer` feature.

use std::error::Error;
use std::fmt;
use std::path::{Path, PathBuf};
use std::time::Duration;

use gstreamer as gst;
use gstreamer_pbutils as pbutils;
use pbutils::prelude::*;

use crate::tracks::{MediaFile, Track, TrackKind};

/// Why a file could not be probed.
#[derive(Debug)]
pub enum ProbeError {
    /// GStreamer failed to initialise.
    Init(gst::glib::Error),
    /// The path could not be turned into a `file://` URI.
    Path(PathBuf),
    /// Discoverer failed to read the file.
    Discover(PathBuf, gst::glib::Error),
}

impl fmt::Display for ProbeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Init(e) => write!(f, "GStreamer failed to initialise: {e}"),
            Self::Path(p) => write!(f, "{} cannot be turned into a file URI", p.display()),
            Self::Discover(p, e) => write!(f, "could not read {}: {e}", p.display()),
        }
    }
}

impl Error for ProbeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Init(e) | Self::Discover(_, e) => Some(e),
            Self::Path(_) => None,
        }
    }
}

/// Reads tracks out of files. Create one and reuse it for a whole folder.
pub struct Prober {
    discoverer: pbutils::Discoverer,
}

impl Prober {
    /// Creates a prober that gives up on any one file after `timeout`.
    ///
    /// # Errors
    ///
    /// Fails when GStreamer cannot be initialised or the Discoverer cannot be
    /// created.
    pub fn new(timeout: Duration) -> Result<Self, ProbeError> {
        gst::init().map_err(ProbeError::Init)?;
        let timeout =
            gst::ClockTime::from_nseconds(u64::try_from(timeout.as_nanos()).unwrap_or(u64::MAX));
        let discoverer = pbutils::Discoverer::new(timeout).map_err(ProbeError::Init)?;
        Ok(Self { discoverer })
    }

    /// Reads the audio and subtitle tracks of the file at `path`. Tracks are
    /// numbered from 1 per kind, in the order Discoverer reports them.
    ///
    /// Language tags come back from GStreamer as two-letter codes where one
    /// exists, so they are turned back into the three-letter bibliographic codes
    /// Matroska stores, such as `jpn`, `eng` and `fre`.
    ///
    /// # Errors
    ///
    /// Fails when the path has no absolute form or the file cannot be read.
    pub fn probe(&self, path: &Path) -> Result<MediaFile, ProbeError> {
        let absolute = std::fs::canonicalize(path).map_err(|_| ProbeError::Path(path.into()))?;
        let uri = gst::glib::filename_to_uri(&absolute, None)
            .map_err(|_| ProbeError::Path(path.into()))?;
        let info = self
            .discoverer
            .discover_uri(&uri)
            .map_err(|e| ProbeError::Discover(path.into(), e))?;

        let audio = info
            .audio_streams()
            .iter()
            .enumerate()
            .map(|(i, s)| track(TrackKind::Audio, i + 1, s.language(), s))
            .collect();
        let subtitles = info
            .subtitle_streams()
            .iter()
            .enumerate()
            .map(|(i, s)| track(TrackKind::Subtitle, i + 1, s.language(), s))
            .collect();
        Ok(MediaFile {
            path: path.into(),
            audio,
            subtitles,
        })
    }
}

fn track(
    kind: TrackKind,
    number: usize,
    language: Option<gst::glib::GString>,
    stream: &impl IsA<pbutils::DiscovererStreamInfo>,
) -> Track {
    let tags = stream.tags();
    let language = language
        .map(|l| l.to_string())
        .or_else(|| {
            tags.as_ref()
                .and_then(|t| t.get::<gst::tags::LanguageCode>())
                .map(|v| v.get().to_owned())
        })
        .unwrap_or_default();
    let language = gstreamer_tag::language_codes::language_code_iso_639_2b(&language)
        .map_or(language, ToString::to_string);
    let title = tags
        .as_ref()
        .and_then(|t| t.get::<gst::tags::Title>())
        .map(|v| v.get().to_owned())
        .unwrap_or_default();
    Track {
        kind,
        number,
        language,
        title,
    }
}
