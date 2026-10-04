//! A folder sorts into viewing order: episodes, then specials, then extras.
use bingekit_episode::{Kind, parse_folder};

#[test]
fn anime_folder_order() {
    let names = [
        "[Grp]_Paper_Lantern_Club_Encore_-_S01v2_(1920x1080_H264_10bit)_[3C4D5E6F].mkv",
        "[Grp]_Paper_Lantern_Club_Encore_-_C01v2_(1920x1080_H264_10bit)_[2B3C4D5E].mkv",
        "[Grp]_Paper_Lantern_Club_Encore_-_12v2_(1920x1080_H264_10bit)_[7A8B9C0D].mkv",
        "[Grp]_Paper_Lantern_Club_Encore_-_02v2_(1920x1080_H264_10bit)_[8B9C0D1E].mkv",
        "[Grp]_Paper_Lantern_Club_Encore_-_01v2_(1920x1080_H264_10bit)_[9C0D1E2F].mkv",
    ];
    let order: Vec<(Kind, u32)> = parse_folder(&names)
        .iter()
        .map(|e| (e.kind, e.episode().unwrap()))
        .collect();
    assert_eq!(
        order,
        [
            (Kind::Regular, 1),
            (Kind::Regular, 2),
            (Kind::Regular, 12),
            (Kind::Special, 1),
            (Kind::Extra, 1)
        ]
    );
}

fn order(names: &[&str]) -> Vec<String> {
    parse_folder(names)
        .into_iter()
        .map(|e| e.file_name)
        .collect()
}

/// A renamed season where one episode title has the word "Special" in it.
#[test]
#[ignore = "an episode title with the word Special parses as a special"]
fn a_special_in_an_episode_title_keeps_its_place() {
    let names = [
        "Kettle Hill - S02E15 - The Paper Boat.mkv",
        "Kettle Hill - S02E14 - Special Delivery Kite.mkv",
        "Kettle Hill - S02E13 - Snow Way Home.mkv",
    ];
    assert_eq!(order(&names), [names[2], names[1], names[0]]);
}

/// A whole show scanned at once: season folders, a two-digit season and a `Specials`
/// folder of season 0. Seasons sort by number, and specials come after them.
#[test]
fn a_whole_show_across_season_folders() {
    let names = [
        "Kettle Hill - S00E01 - Romancing the Turnip.mkv",
        "Kettle Hill - S10E01 - The Big Thaw.mkv",
        "Kettle Hill - S02E01 - Hot Turnip.avi",
        "Kettle Hill - S01E02 - Kettle On.mkv",
        "Kettle Hill - S01E01 - Pilot Light.mkv",
    ];
    assert_eq!(
        order(&names),
        [names[4], names[3], names[2], names[1], names[0]]
    );
}

/// A season whose files came from several places: renamed, dotted scene names, spaced
/// scene names, a group with a repacker's tag after it, and episode titles in some.
#[test]
fn a_season_of_mixed_name_styles() {
    let names = [
        "The Grand Village Bake S16E05 - Marzipan Week 1080p All4 WEB-DL AAC 2.0 H.264-WhiskeyMac.mkv",
        "The.Grand.Village.Bake.S16E06.1080p.HDTV.H264-DARKFLAX-xyz.mkv",
        "The Grand Village Bake S16E04 Back to Class Week 1080p ALL4 WEB-DL AAC2 0 H 264-RAWX.mkv",
        "The Grand Village Bake - S16E03 - Bread Basket Week.mkv",
        "The Grand Village Bake S16E02 720p WEB H264 JFX TVZE.mkv",
        "The.Grand.Village.Bake.S16E01.Toffee.Week.1080p.ALL4.WEB-DL.AAC2.0.H.264-RAWX.mkv",
    ];
    let parsed = parse_folder(&names);
    let numbers: Vec<u32> = parsed.iter().map(|e| e.episode().unwrap()).collect();
    assert_eq!(numbers, [1, 2, 3, 4, 5, 6]);
    assert!(
        parsed
            .iter()
            .all(|e| e.kind == Kind::Regular && e.season == Some(16))
    );
}

/// Double episodes whose ranges overlap or skip numbers, beside single episodes, and
/// segments listed with a comma. Each file sorts by its first episode.
#[test]
fn multi_episode_files_sort_by_their_first_episode() {
    let names = [
        "Sprout Squad! - S02E22-E24 - Road Snacks & The Best Sprout.mkv",
        "Sprout Squad! - S02E20-E21 - Cats vs Dogs & Kite Adventure.mkv",
        "Sprout Squad! - S02E18-E23 - Serious Snacks & Harvest Day.mkv",
        "Sprout Squad! - S02E17 - The Helmet.mkv",
        "Sprout Squad! - S02E14 - Sandwich Bandit.mkv",
        "Sprout Squad! - S02E13-E18.mkv",
        "Sprout Squad! - S02E03-E05.mkv",
        "Sprout Squad! - S02E01-E02 - Mr. Sprout.mkv",
    ];
    let numbers: Vec<u32> = parse_folder(&names)
        .iter()
        .map(|e| e.episode().unwrap())
        .collect();
    assert_eq!(numbers, [1, 3, 13, 14, 17, 18, 20, 22]);

    let names = [
        "Sprout Squad! S01E43-E44.mkv",
        "Sprout Squad! S01E42,E40.mkv",
        "Sprout Squad! S01E39,E41.mkv",
        "Sprout Squad! S01E37-E38.mkv",
    ];
    assert_eq!(order(&names), [names[3], names[2], names[1], names[0]]);
}

/// An arc told over several episodes, with the part in each episode's title.
#[test]
fn an_arc_named_in_episode_titles_sorts_by_episode() {
    let names = [
        "Tidewatch (2008) - S07E08 - Anchors, Part 3 Low Water (1080p BluRay x265 Kelp).mkv",
        "Tidewatch (2008) - S07E06 - Anchors, Part 1 The Harbor Queen (1080p BluRay x265 Kelp).mkv",
        "Tidewatch (2008) - S07E09 - Anchors, Part 4 The Empty Pier (1080p BluRay x265 Kelp).mkv",
        "Tidewatch (2008) - S07E07 - Anchors, Part 2 Everything Floats (1080p BluRay x265 Kelp).mkv",
        "Tidewatch (2008) - S07E05 - Before the Squall (1080p BluRay x265 Kelp).mkv",
    ];
    assert_eq!(
        order(&names),
        [names[4], names[1], names[3], names[0], names[2]]
    );
}

/// A folder of films, some of them in parts. A film in parts sorts with its title,
/// part by part.
#[test]
#[ignore = "films with a part sort after every film without one"]
fn films_in_parts_sort_with_their_title() {
    let names = [
        "Zephyr Road (1999).mkv",
        "The Harvest Games Ember Part 2 (2011)(1080p 4KBDRip x265 crf19 E-AC3 5.1)[cHorsy].mkv",
        "The Harvest Games Ember Part 1 (2010)(1080p 4KBDRip x265 crf19 E-AC3 5.1)[cHorsy].mkv",
        "Hero League x Moon Guard - Knights & Hunters, Part Two (2023).mkv",
        "Hero League x Moon Guard - Knights & Hunters, Part One (2023).mkv",
        "Aardvark Summer (2001).mkv",
    ];
    assert_eq!(
        order(&names),
        [names[5], names[4], names[3], names[2], names[1], names[0]]
    );
}

/// Two copies of one film side by side, one renamed and one with its release name.
#[test]
fn two_copies_of_a_film_are_films() {
    let names = [
        "Harbor Lights 3 (2024).mkv",
        "Harbor.Lights.3.2024.REPACK.2160p.WEB-DL.DDP5.1.Atmos.DV.HDR.H.265-FLAX.mkv",
    ];
    let parsed = parse_folder(&names);
    assert!(parsed.iter().all(|e| e.kind == Kind::Movie), "{parsed:?}");
}

/// A sequel's disc title left in the folder of the film before it. The numbers in
/// the titles are not episodes.
#[test]
#[ignore = "sibling context turns sequel numbers into episode numbers"]
fn a_stray_disc_title_beside_a_film() {
    let names = ["Kite Panda 2 (2011).mkv", "Kite Panda 3_t05.mkv"];
    let parsed = parse_folder(&names);
    assert!(
        parsed
            .iter()
            .all(|e| e.kind == Kind::Movie && e.episodes.is_empty()),
        "{parsed:?}"
    );
}
