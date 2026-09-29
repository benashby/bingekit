//! Pairings and the per-file track choice built from them.

use bingekit_library::{
    MediaFile, Pairing, Track, TrackKind, choose_tracks, default_pairings, summary,
};

fn file(name: &str, audio: &[(&str, &str)], subtitles: &[(&str, &str)]) -> MediaFile {
    let tracks = |kind, list: &[(&str, &str)]| {
        list.iter()
            .enumerate()
            .map(|(i, (lang, title))| Track::new(kind, i + 1, lang, title))
            .collect()
    };
    MediaFile {
        path: name.into(),
        audio: tracks(TrackKind::Audio, audio),
        subtitles: tracks(TrackKind::Subtitle, subtitles),
    }
}

fn dual_audio(name: &str) -> MediaFile {
    file(
        name,
        &[("jpn", ""), ("eng", "")],
        &[("eng", "Full Subtitles")],
    )
}

#[test]
fn default_pairings_keep_mpv_launcher_order() {
    let ids: Vec<_> = default_pairings().into_iter().map(|p| p.id).collect();
    assert_eq!(ids, ["jpn_eng", "eng_none", "jpn_none", "eng_eng"]);
}

#[test]
fn a_pairing_fits_only_when_both_languages_exist() {
    let pairings = default_pairings();
    let english_only = file("a.mkv", &[("eng", "")], &[]);
    assert!(!pairings[0].fits(&english_only));
    assert!(pairings[1].fits(&english_only));
    assert_eq!(pairings[1].tracks_in(&english_only), Some((1, None)));
    assert_eq!(
        pairings[0].tracks_in(&dual_audio("b.mkv")),
        Some((1, Some(1)))
    );
}

#[test]
fn the_first_fitting_pairing_wins() {
    let choices = choose_tracks(&[dual_audio("ep01.mkv")], &default_pairings());
    assert_eq!(choices[0].pairing, Some(0));
    assert_eq!((choices[0].audio, choices[0].subtitles), (Some(1), Some(1)));
}

#[test]
fn reordering_changes_the_choice() {
    let mut pairings = default_pairings();
    pairings.swap(0, 1);
    let choices = choose_tracks(&[dual_audio("ep01.mkv")], &pairings);
    assert_eq!(pairings[choices[0].pairing.unwrap()].id, "eng_none");
    assert_eq!((choices[0].audio, choices[0].subtitles), (Some(2), None));
}

#[test]
fn a_file_that_fits_nothing_gets_its_first_tracks() {
    let french = file("fr.mkv", &[("fre", "")], &[("fre", ""), ("ger", "")]);
    let silent = file("silent.mkv", &[], &[]);
    let choices = choose_tracks(&[french, silent], &default_pairings());
    assert_eq!(choices[0].pairing, None);
    assert_eq!((choices[0].audio, choices[0].subtitles), (Some(1), Some(1)));
    assert_eq!((choices[1].audio, choices[1].subtitles), (None, None));
}

#[test]
fn each_file_is_chosen_on_its_own() {
    let files = [
        dual_audio("ep01.mkv"),
        file("ep02.mkv", &[("eng", "")], &[]),
        file("ep03.mkv", &[("fre", "")], &[]),
    ];
    let choices = choose_tracks(&files, &default_pairings());
    let picked: Vec<_> = choices.iter().map(|c| c.pairing).collect();
    assert_eq!(picked, [Some(0), Some(1), None]);
}

#[test]
fn summary_counts_in_pairing_order_then_fallback() {
    let pairings = vec![
        Pairing::new("eng_none", "eng", None, "English"),
        Pairing::new("jpn_none", "jpn", None, "Japanese"),
    ];
    let files = [
        file("1.mkv", &[("jpn", "")], &[]),
        file("2.mkv", &[("eng", "")], &[]),
        file("3.mkv", &[("jpn", "")], &[]),
        file("4.mkv", &[("fre", "")], &[]),
    ];
    let rows = summary(&choose_tracks(&files, &pairings), &pairings);
    assert_eq!(
        rows,
        [
            ("English".to_owned(), 1),
            ("Japanese".to_owned(), 2),
            ("Fallback".to_owned(), 1)
        ]
    );
}
