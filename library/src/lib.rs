//! Library logic for playing a folder of episodes: finding the video files,
//! describing their audio and subtitle tracks, and picking tracks for each file
//! from a ranked list of language pairings.
//!
//! Episode order comes from `bingekit-episode`. Two optional features read a
//! real file's tracks: `matroska` reads a Matroska file's headers in pure
//! Rust, and `gstreamer` asks GStreamer's Discoverer about any format it can
//! play. Without either, the caller builds each [`MediaFile`] itself.
//!
//! ```
//! use bingekit_library::{MediaFile, Track, TrackKind, default_pairings, choose_tracks};
//!
//! let file = MediaFile {
//!     path: "ep01.mkv".into(),
//!     audio: vec![
//!         Track::new(TrackKind::Audio, 1, "jpn", "Japanese"),
//!         Track::new(TrackKind::Audio, 2, "eng", "English"),
//!     ],
//!     subtitles: vec![Track::new(TrackKind::Subtitle, 1, "eng", "Full Subtitles")],
//! };
//! let choices = choose_tracks(&[file], &default_pairings());
//! assert_eq!(choices[0].audio, Some(1));
//! assert_eq!(choices[0].subtitles, Some(1));
//! ```

// Runs the README example as a doctest, so the README cannot drift from the code.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
pub struct ReadmeDoctests;

#[cfg(feature = "matroska")]
pub mod matroska;
pub mod playlist;
pub mod priority;
#[cfg(feature = "gstreamer")]
pub mod probe;
pub mod scan;
pub mod tracks;

pub use playlist::{Choice, choose_tracks, summary};
pub use priority::{Pairing, default_pairings};
pub use scan::{ScanOptions, is_video_file, scan};
pub use tracks::{MediaFile, Track, TrackKind, effective_language};
