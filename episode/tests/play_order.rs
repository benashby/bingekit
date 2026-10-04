//! A folder plays one file per episode, from the release playback started in.
//!
//! The shows and release groups are made up. Each folder keeps the structure of one
//! that came up in real use.
use bingekit_episode::{next_after, parse_folder, play_order};
use proptest::prelude::*;

fn names(order: &[bingekit_episode::Episode]) -> Vec<&str> {
    order.iter().map(|e| e.file_name.as_str()).collect()
}

fn next(folder: &[&str], current: &str) -> Option<String> {
    next_after(folder, current).map(|e| e.file_name)
}

/// Two fansub releases of a whole season side by side: one with two audio tracks and
/// chapters (`10bit` in its name), one with a single untagged track. Starting in one
/// must never wander into the other.
const TWO_RELEASES: [&str; 9] = [
    "[Grp]_Paper_Lantern_Club_-_01_(1920x1080_H264_10bit)_[1A2B3C4D].mkv",
    "[Grp]_Paper_Lantern_Club_-_02_(1920x1080_H264_10bit)_[2B3C4D5E].mkv",
    "[Grp]_Paper_Lantern_Club_-_03_(1920x1080_H264_10bit)_[3C4D5E6F].mkv",
    "[Grp]_Paper_Lantern_Club_-_S01v2_(1920x1080_H264_10bit)_[4D5E6F70].mkv",
    "[Grp]_Paper_Lantern_Club_-_C01_(1920x1080_H264_10bit)_[5E6F7081].mkv",
    "[Unknown]_Paper_Lantern_Club_-_01_(1920x1080_H264)_[6F708192].mkv",
    "[Unknown]_Paper_Lantern_Club_-_02_(1920x1080_H264)_[708192A3].mkv",
    "[Unknown]_Paper_Lantern_Club_-_03_(1920x1080_H264)_[8192A3B4].mkv",
    "[Unknown]_Paper_Lantern_Club_-_S01_(1920x1080_H264)_[92A3B4C5].mkv",
];

#[test]
fn two_releases_side_by_side_stay_apart() {
    let grp = TWO_RELEASES[0];
    assert_eq!(
        names(&play_order(&TWO_RELEASES, Some(grp))),
        [
            TWO_RELEASES[0],
            TWO_RELEASES[1],
            TWO_RELEASES[2],
            TWO_RELEASES[3],
            TWO_RELEASES[4],
        ],
        "episodes, the special and the extra, all from the starting release"
    );
    assert_eq!(next(&TWO_RELEASES, grp).as_deref(), Some(TWO_RELEASES[1]));
    assert_eq!(
        next(&TWO_RELEASES, TWO_RELEASES[2]).as_deref(),
        Some(TWO_RELEASES[3])
    );

    let unknown = TWO_RELEASES[5];
    assert_eq!(
        next(&TWO_RELEASES, unknown).as_deref(),
        Some(TWO_RELEASES[6])
    );
    assert_eq!(
        names(&play_order(&TWO_RELEASES, Some(unknown))),
        [
            TWO_RELEASES[5],
            TWO_RELEASES[6],
            TWO_RELEASES[7],
            TWO_RELEASES[8],
            TWO_RELEASES[4],
        ],
        "the other release, with the extra only the first one has"
    );
    // The listing still shows every file.
    assert_eq!(parse_folder(&TWO_RELEASES).len(), TWO_RELEASES.len());
}

#[test]
fn no_start_plays_the_first_episodes_release() {
    let order = play_order(&TWO_RELEASES, None);
    assert_eq!(order.len(), 5);
    assert!(order.iter().all(|e| e.file_name.starts_with("[Grp]")));
}

#[test]
fn a_v2_replaces_the_file_it_fixed() {
    let folder = [
        "[Subs] Tidewatch - 01 (1080p) [0A1B2C3D].mkv",
        "[Subs] Tidewatch - 02 (1080p) [1B2C3D4E].mkv",
        "[Subs] Tidewatch - 02v2 (1080p) [2C3D4E5F].mkv",
        "[Subs] Tidewatch - 03 (1080p) [3D4E5F60].mkv",
    ];
    assert_eq!(
        names(&play_order(&folder, Some(folder[0]))),
        [folder[0], folder[2], folder[3]]
    );
    // Starting on the original still plays it: the viewer picked that file.
    assert_eq!(
        names(&play_order(&folder, Some(folder[1]))),
        [folder[0], folder[1], folder[3]]
    );
}

/// A TV season whose episodes came from two groups, with one episode only the second
/// group released.
const MIXED_SEASON: [&str; 7] = [
    "The.Quiet.Coast.S01E01.1080p.WEB.h264-GRPA.mkv",
    "The.Quiet.Coast.S01E02.1080p.WEB.h264-GRPA.mkv",
    "The.Quiet.Coast.S01E04.1080p.WEB.h264-GRPA.mkv",
    "The.Quiet.Coast.S01E01.720p.HDTV.x264-GRPB.mkv",
    "The.Quiet.Coast.S01E02.720p.HDTV.x264-GRPB.mkv",
    "The.Quiet.Coast.S01E03.720p.HDTV.x264-GRPB.mkv",
    "The.Quiet.Coast.S01E04.720p.HDTV.x264-GRPB.mkv",
];

#[test]
fn a_season_from_two_groups_plays_through_its_gap() {
    assert_eq!(
        names(&play_order(&MIXED_SEASON, Some(MIXED_SEASON[0]))),
        [
            MIXED_SEASON[0],
            MIXED_SEASON[1],
            MIXED_SEASON[5],
            MIXED_SEASON[2]
        ],
        "episode 3 from the only group that has it, then back"
    );
    assert_eq!(
        next(&MIXED_SEASON, MIXED_SEASON[1]).as_deref(),
        Some(MIXED_SEASON[5])
    );
    // From the stand-in, the next episode follows the release that was playing.
    assert_eq!(
        next(&MIXED_SEASON, MIXED_SEASON[5]).as_deref(),
        Some(MIXED_SEASON[6])
    );
    assert_eq!(
        names(&play_order(&MIXED_SEASON, Some(MIXED_SEASON[3]))),
        [
            MIXED_SEASON[3],
            MIXED_SEASON[4],
            MIXED_SEASON[5],
            MIXED_SEASON[6]
        ]
    );
}

#[test]
fn one_group_in_two_resolutions_follows_the_one_playing() {
    let folder = [
        "[Subs] Tidewatch - 01 (1080p) [0A1B2C3D].mkv",
        "[Subs] Tidewatch - 01 (720p) [1B2C3D4E].mkv",
        "[Subs] Tidewatch - 02 (1080p) [2C3D4E5F].mkv",
        "[Subs] Tidewatch - 02 (720p) [3D4E5F60].mkv",
    ];
    assert_eq!(next(&folder, folder[1]).as_deref(), Some(folder[3]));
    assert_eq!(next(&folder, folder[0]).as_deref(), Some(folder[2]));
}

#[test]
fn a_double_episode_file_stands_in_for_two_singles() {
    let folder = [
        "Show.Name.S01E01E02.1080p.WEB-GRPA.mkv",
        "Show.Name.S01E03.1080p.WEB-GRPA.mkv",
        "Show.Name.S01E01.720p.HDTV-GRPB.mkv",
        "Show.Name.S01E02.720p.HDTV-GRPB.mkv",
    ];
    assert_eq!(
        names(&play_order(&folder, Some(folder[0]))),
        [folder[0], folder[1]],
        "the double covers 1 and 2"
    );
    assert_eq!(
        names(&play_order(&folder, Some(folder[2]))),
        [folder[2], folder[3], folder[1]],
        "the singles cover them instead"
    );
}

#[test]
fn extras_and_unnumbered_files_are_never_merged() {
    let folder = [
        "[Grp] Show - 01 [1080p].mkv",
        "[Grp] Show - NCOP1 [1080p].mkv",
        "[Alt] Show - NCOP1 [1080p].mkv",
        "[Grp] Show - C01 [1080p].mkv",
        "Behind the scenes.mkv",
    ];
    let order = play_order(&folder, Some(folder[0]));
    assert_eq!(order.len(), folder.len(), "{:?}", names(&order));
}

#[test]
fn the_end_of_a_folder_has_nothing_next() {
    let folder = ["[Grp] Show - 01 [1080p].mkv", "[Grp] Show - 02 [1080p].mkv"];
    assert_eq!(next(&folder, folder[0]).as_deref(), Some(folder[1]));
    assert_eq!(next(&folder, folder[1]), None);
    assert_eq!(
        next(&folder, "[Grp] Show - 03 [1080p].mkv"),
        None,
        "not in the folder"
    );
    assert!(play_order::<&str>(&[], None).is_empty());
}

proptest! {
    /// For any mix of releases and any starting file, the start plays, nothing plays
    /// twice, and the order is the folder's viewing order with files left out.
    #[test]
    fn keeps_the_start_and_the_viewing_order(
        groups in proptest::collection::vec(prop_oneof!["Grp", "Alt", "Subs"], 1..4),
        episodes in proptest::collection::vec(1u32..13, 1..12),
        res in prop_oneof!["1080p", "720p"],
        pick in any::<prop::sample::Index>(),
    ) {
        let folder: Vec<String> = episodes
            .iter()
            .zip(groups.iter().cycle())
            .map(|(n, g)| format!("[{g}] Made-up Show - {n:02} ({res}) [00{n:06}].mkv"))
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();
        let start = pick.get(&folder).clone();
        let order = play_order(&folder, Some(&start));
        let all: Vec<String> = parse_folder(&folder).into_iter().map(|e| e.file_name).collect();
        prop_assert!(order.iter().any(|e| e.file_name == start));
        let mut at = 0;
        for e in &order {
            let i = all[at..].iter().position(|n| *n == e.file_name);
            prop_assert!(i.is_some(), "{} out of order", e.file_name);
            at += i.unwrap() + 1;
        }
        let numbers: Vec<u32> = order.iter().map(|e| e.episodes[0]).collect();
        let unique: std::collections::BTreeSet<u32> = numbers.iter().copied().collect();
        prop_assert_eq!(numbers.len(), unique.len(), "an episode played twice");
    }
}
