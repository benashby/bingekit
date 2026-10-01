//! Track languages, including the mislabelled tags common in fan encodes.

use bingekit_library::{MediaFile, Track, TrackKind, effective_language};

/// One episode of a made-up anime release. The audio tags say `und`, so only
/// the titles tell the two languages apart, and there are three English
/// subtitle tracks.
fn fan_encode_episode(n: usize) -> MediaFile {
    MediaFile {
        path: format!("ep{n:02}.mkv").into(),
        audio: vec![
            Track::new(TrackKind::Audio, 1, "und", "FLAC 2.0 (Japanese)"),
            Track::new(TrackKind::Audio, 2, "und", "FLAC 5.1 (English)"),
        ],
        subtitles: vec![
            Track::new(TrackKind::Subtitle, 1, "eng", "Full Subtitles [Grp]"),
            Track::new(TrackKind::Subtitle, 2, "eng", "Signs & Songs"),
            Track::new(TrackKind::Subtitle, 3, "eng", "Full Subtitles [Other]"),
        ],
    }
}

#[test]
fn title_overrides_a_wrong_or_missing_tag() {
    let ep = fan_encode_episode(1);
    assert_eq!(ep.audio[0].language().as_deref(), Some("jpn"));
    assert_eq!(ep.audio[1].language().as_deref(), Some("eng"));
    for sub in &ep.subtitles {
        assert_eq!(sub.language().as_deref(), Some("eng"), "{}", sub.title);
    }
}

#[test]
fn both_usual_choices_exist_in_every_episode() {
    for n in 1..=3 {
        let ep = fan_encode_episode(n);
        assert!(ep.has_language(TrackKind::Audio, "eng"));
        assert!(ep.has_language(TrackKind::Audio, "jpn"));
        assert!(ep.has_language(TrackKind::Subtitle, "eng"));
    }
}

#[test]
fn find_returns_the_first_matching_track() {
    let ep = fan_encode_episode(1);
    let sub = ep.find(TrackKind::Subtitle, "eng").unwrap();
    assert_eq!(sub.number, 1);
    assert!(ep.find(TrackKind::Subtitle, "fre").is_none());
}

#[test]
fn tag_is_used_when_the_title_says_nothing() {
    assert_eq!(effective_language("fre", "Stereo").as_deref(), Some("fre"));
    assert_eq!(effective_language("ENG", "").as_deref(), Some("eng"));
    assert_eq!(effective_language("", ""), None);
}

#[test]
fn two_letter_tags_match_three_letter_codes() {
    assert_eq!(effective_language("ja", "").as_deref(), Some("jpn"));
    assert_eq!(effective_language("en", "").as_deref(), Some("eng"));
}

#[test]
fn title_words_are_matched_whole() {
    assert_eq!(effective_language("ben", "Bengali").as_deref(), Some("ben"));
    assert_eq!(effective_language("und", "ENG dub").as_deref(), Some("eng"));
    assert_eq!(effective_language("und", "日本語").as_deref(), Some("jpn"));
}

#[test]
fn japanese_wins_when_a_title_names_both() {
    assert_eq!(
        effective_language("und", "English/Japanese dual").as_deref(),
        Some("jpn")
    );
}

#[test]
fn signs_tracks_are_told_apart_by_title() {
    let ep = fan_encode_episode(1);
    let signs: Vec<_> = ep.subtitles.iter().map(Track::is_signs).collect();
    assert_eq!(signs, [false, true, false]);
    assert!(Track::new(TrackKind::Subtitle, 1, "eng", "English [Forced]").is_signs());
    assert!(!Track::new(TrackKind::Subtitle, 1, "eng", "Design notes").is_signs());
    assert_eq!(ep.find_dialogue("eng").map(|t| t.number), Some(1));
    assert_eq!(ep.find_signs("eng").map(|t| t.number), Some(2));
}
