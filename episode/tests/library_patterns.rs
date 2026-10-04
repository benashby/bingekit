//! File names in the shapes a real home library holds, parsed one at a time.
//!
//! The shows, films, episode titles and release groups are made up. Each name keeps
//! the structure of a file found in a library of a few thousand episodes and films:
//! the separators, brackets, tag order and numbering are as found.
use bingekit_episode::{Episode, Kind, parse};

/// What one name should parse to.
struct Row {
    name: &'static str,
    kind: Kind,
    season: Option<u32>,
    episodes: &'static [u32],
    part: Option<u32>,
    group: Option<&'static str>,
}

const fn row(name: &'static str, kind: Kind, season: Option<u32>, episodes: &'static [u32]) -> Row {
    Row {
        name,
        kind,
        season,
        episodes,
        part: None,
        group: None,
    }
}

const fn ep(name: &'static str, season: u32, episodes: &'static [u32]) -> Row {
    row(name, Kind::Regular, Some(season), episodes)
}

const fn film(name: &'static str) -> Row {
    row(name, Kind::Movie, None, &[])
}

impl Row {
    const fn part(mut self, part: u32) -> Self {
        self.part = Some(part);
        self
    }

    const fn group(mut self, group: &'static str) -> Self {
        self.group = Some(group);
        self
    }
}

fn check(rows: &[Row]) {
    let mut wrong = Vec::new();
    for row in rows {
        let got: Episode = parse(row.name);
        let fields = (
            got.kind,
            got.season,
            got.episodes.as_slice(),
            got.part,
            got.release_group.as_deref(),
        );
        let want = (row.kind, row.season, row.episodes, row.part, row.group);
        if fields != want {
            wrong.push(format!("{}\n   got {fields:?}\n  want {want:?}", row.name));
        }
    }
    assert!(wrong.is_empty(), "\n{}", wrong.join("\n"));
}

/// `Show - SxxEyy - Episode Title.ext`, the shape a library manager renames to. Most
/// of a library looks like this, in mkv, mp4 and avi.
#[test]
fn renamed_episodes() {
    check(&[
        ep("Brass Harbor - S02E07 - The Ferryman's Ledger.mkv", 2, &[7]),
        ep("Brass Harbor - S03E14 - Gull vs. Me-Gull.mp4", 3, &[14]),
        ep(
            "Kettle Hill - S06E13 - Soup Days II - The Long Ladle.avi",
            6,
            &[13],
        ),
        // The show's own name has a dash in it.
        ep(
            "Paper Lantern Club - Night Shift - S03E25 - The Lantern Parade.mkv",
            3,
            &[25],
        ),
        ep(
            "Kettle Hill - Reheated - S02E04 - A Pie on Every Plate - The Reheat.mkv",
            2,
            &[4],
        ),
        // Numbers, punctuation and odd characters inside the episode title.
        ep("Brass Harbor - S07E01 - Harbortown 2.mkv", 7, &[1]),
        ep(
            "Storm Riders Zeta - S05E13 - Say Farewell, 23.mkv",
            5,
            &[13],
        ),
        ep("Tidewatch - S04E20 - Gull 4 - Other Ending.mp4", 4, &[20]),
        ep(
            "Storm Riders Zeta - S01E01 - A Calm Reward - Who Wins the 5,000,000 Coins.mkv",
            1,
            &[1],
        ),
        ep(
            "Storm Riders Zeta - S01E05 - Duel on Cloud Island! Kai vs. the Tempest.mkv",
            1,
            &[5],
        ),
        ep("Willow Lane - S02E01 - Hmmmmmmmm!.mkv", 2, &[1]),
        ep(
            "Willow Lane - S02E06 - Ah Ah Ah... It's Tricks.mkv",
            2,
            &[6],
        ),
        ep("Willow Lane - S02E12 - Salt & Pepper.mkv", 2, &[12]),
        ep(
            "Willow Lane - S01E08 - I Don\u{2019}t Want to Be Anything but Brave.mkv",
            1,
            &[8],
        ),
        // A two-digit season beyond the first few.
        ep("Castaway Isle - S47E06 - Feel the Tide.mkv", 47, &[6]),
        // No episode title at all, and a year after the show's name.
        ep("Moss Garden (2026) - S01E08.mkv", 1, &[8]),
        // A library manager's quality suffix after the episode title.
        ep(
            "Willow Lane - S01E07 - I\u{2019}ve Been Making Plans, but Each Plan Has Helped Me HDTV-720p Proper.mkv",
            1,
            &[7],
        ),
        // A year after the show, and a bracket of release tags whose last word is the
        // encoder rather than a group.
        ep(
            "Tidewatch (2008) - S03E05 - Seasick at Sea (1080p BluRay x265 Kelp).mkv",
            3,
            &[5],
        ),
    ]);
}

/// Story arcs split across episodes carry the part in the episode title. The episode
/// number stays the episode; the part is reported as found.
#[test]
fn parts_named_in_episode_titles() {
    check(&[
        ep(
            "Brass Harbor - S02E26 - Signals from the Reef (1).mkv",
            2,
            &[26],
        ),
        ep(
            "Brass Harbor - S02E27 - Signals from the Reef (2).mkv",
            2,
            &[27],
        ),
        ep(
            "Brass Harbor - S03E11 - The Long Night - The Shadow (2).mkv",
            3,
            &[11],
        ),
        ep(
            "Tidewatch (2008) - S07E08 - Anchors, Part 3 Low Water (1080p BluRay x265 Kelp).mkv",
            7,
            &[8],
        )
        .part(3),
        ep("Tidewatch - S05E23 - Reunion (Part 1).mp4", 5, &[23]).part(1),
    ]);
}

/// Season 0, in a `Specials` folder.
#[test]
fn season_zero_specials() {
    check(&[
        row(
            "Kettle Hill - S00E04 - Kettle Hill 10th Anniversary.mkv",
            Kind::Special,
            Some(0),
            &[4],
        ),
        row(
            "Kettle Hill - S00E03 - Kettle Hill 2 - Follow That Kite.mkv",
            Kind::Special,
            Some(0),
            &[3],
        ),
        row(
            "Kettle Hill - S00E03 - Eat This Pebble.avi",
            Kind::Special,
            Some(0),
            &[3],
        ),
    ]);
}

/// Files holding several episodes. A range is read as every episode in it.
#[test]
fn multi_episode_files() {
    check(&[
        ep(
            "Brass Harbor - S07E15-E16 - Brass Harbor - The Opera.mkv",
            7,
            &[15, 16],
        ),
        ep(
            "Sprout Squad! - S02E20-E21 - Cats vs Dogs & Kite Adventure.mkv",
            2,
            &[20, 21],
        ),
        ep("Sprout Squad! - S02E03-E05.mkv", 2, &[3, 4, 5]),
        ep("Sprout Squad! S01E05-E06.mkv", 1, &[5, 6]),
        ep(
            "Tidewatch - S05E28-E32 - Change the Tide.mp4",
            5,
            &[28, 29, 30, 31, 32],
        ),
        ep(
            "Tidewatch (2008) - S07E14-E15 - The More You Row & The Row You Know (1080p BluRay x265 Kelp).mkv",
            7,
            &[14, 15],
        ),
        // Two segments that aired together under non-adjacent numbers. The name
        // can't say whether the episodes between are in the file, so the range is
        // taken whole.
        ep(
            "Sprout Squad! - S02E18-E23 - Serious Snacks & Harvest Day.mkv",
            2,
            &[18, 19, 20, 21, 22, 23],
        ),
    ]);
}

/// Scene release names: dots or spaces, the tags, then `-GROUP`.
#[test]
fn scene_episodes() {
    check(&[
        ep(
            "The.Night.Ferry.S01E02.1080p.10bit.WEBRip.6CH.x265.HEVC-PZA.mkv",
            1,
            &[2],
        )
        .group("PZA"),
        ep("Castaway.Isle.S49E08.1080p.WEB.h264-EDNA.mkv", 49, &[8]).group("EDNA"),
        // A tag after the group, from whoever repackaged it.
        ep("Castaway.Isle.S49E03.1080p.WEB.h264-EDNA-xyz.mkv", 49, &[3]).group("EDNA-xyz"),
        ep(
            "Castaway.Isle.S49E07.Blood.in.the.Water.1080p.AMZN.WEB-DL.DDP5.1.H.264-BLOOMS.mkv",
            49,
            &[7],
        )
        .group("BLOOMS"),
        ep("Castaway.Isle.S49E01.480p.x264-mSX.mkv", 49, &[1]).group("mSX"),
        ep(
            "The.Grand.Village.Bake.S16E01.Toffee.Week.1080p.ALL4.WEB-DL.AAC2.0.H.264-RAWX.mkv",
            16,
            &[1],
        )
        .group("RAWX"),
        ep("The.Grand.Village.Bake.S16E10.1080p.HDTV.H264-DARKFLAX.mkv", 16, &[10])
            .group("DARKFLAX"),
        // The same with spaces, the dots of `DDP5.1` and `H.264` lost.
        ep(
            "Castaway Isle S49E06 The Devils Boots 1080p AMZN WEB-DL DDP5 1 H 264-FLAX.mkv",
            49,
            &[6],
        )
        .group("FLAX"),
        ep(
            "The Grand Village Bake S16E05 - Marzipan Week 1080p All4 WEB-DL AAC 2.0 H.264-WhiskeyMac.mkv",
            16,
            &[5],
        )
        .group("WhiskeyMac"),
        // A bracketed group, and the extension written twice.
        ep("Castaway Isle S49E12 1080p WEB-DL-[Ferryman1980] mkv.mkv", 49, &[12])
            .group("Ferryman1980"),
    ]);
}

/// Films: `Title (Year)` in all its library and release shapes.
#[test]
fn films() {
    check(&[
        film("Harbor Lights (1987).mkv"),
        film("Harbor Lights 2 (2014).mkv"),
        film("Sparrow Street 3 - Graduation Day (2008).mkv"),
        film("Sparrow Street 3 Graduation Day (2008).mkv"),
        film("Sparrow Street - Girls - Rainbow Tides (2014).mkv"),
        film("Sparrow Street Girls \u{2013} Rainbow Tides (2014).mkv"),
        film("Stormbreakers- (2025).mkv"),
        film("Oliva! (1968).mkv"),
        film("Mrs. 'Ollie Goes to Lisbon (1992).mkv"),
        film("Aur\u{e9}lie of the Valley of Mist (1984).mkv"),
        film("Critter\u{e9}mon - The Picture 2000 (1999).mkv"),
        film("Critter\u{e9}mon 4Ever - Glade - Voice of the Pines (2001).mkv"),
        film("Kite Panda 3_t05.mkv"),
        film("Gearwork Knights (2011) REPACK2 (1080p BluRay x265 HEVC 10bit AAC 5.1 Tigrol).mkv"),
        film(
            "The Lord of the Lanterns - The Fellowship of the Wick (2001) Extended (2160p UHD BluRay x265 DV HDR DDP 7.1 English - Weaslee HONK).mkv",
        ),
        film("Whispering Pines (1995) 1080p BDRip x265 AAC 5.1 Toki [SEW].mkv").group("SEW"),
        film("Gearwork.Knights.II.2026.2160p.MA.WEB-DL.TrueHD.Atmos.7.1.DV.HDR10P.H.265-TheBarn.mkv")
            .group("TheBarn"),
        film("Paper.Kites.1997.DUAL.1080p.BluRay.x265.DDP5.1-B3ARD.mkv").group("B3ARD"),
        film("Paper.Kites.2004.REPACK.DUAL.1080p.BluRay.x265.DDP5.1-B3ARD.mkv").group("B3ARD"),
        film(
            "The.Harvest.Games.Ballad.of.Wrens.and.Vipers.2023.Hybrid.1080p.BluRay.REMUX.AVC.TrueHD.7.1.Atmos-PYRAMIDHAT.mkv",
        )
        .group("PYRAMIDHAT"),
        film(
            "The.Harvest.Games.Kindling.2013.PROPER.BluRay.1080p.TrueHD.Atmos.7.1.AVC.HYBRID.REMUX-FrameStore.mkv",
        )
        .group("FrameStore"),
        film(
            "Web-Slinger.Into.the.Web-Verse.2018.REPACK.HYBRID.2160p.BluRay.REMUX.HEVC.DV.TrueHD.Atmos.7.1-Flytes.mkv",
        )
        .group("Flytes"),
        film("Sky.Rover.The.Legend.of.Ash.2026.1080p.PMNTP.WEBRip.AAC2.0.H264-[LEEK].mp4")
            .group("LEEK"),
        // Films in parts keep the part.
        film("Hero League x Moon Guard - Knights & Hunters, Part Two (2023).mkv").part(2),
        film("The.Harvest.Games.Ember-Part.1.2014.REPACK.1080p.BluRay.DTS.X264-EbX.mkv")
            .part(1)
            .group("EbX"),
        film(
            "The.Harvest.Games.Ember.Part.2.2015.BluRay.1080p.TrueHD.Atmos.7.1.AVC.REMUX-FrameStore.mkv",
        )
        .part(2)
        .group("FrameStore"),
    ]);
}

/// The show title comes out clean of the year, the separators and the release tags.
#[test]
fn show_titles() {
    for (name, title) in [
        (
            "Paper Lantern Club - Night Shift - S03E25 - The Lantern Parade.mkv",
            "Paper Lantern Club",
        ),
        ("Moss Garden (2026) - S01E08.mkv", "Moss Garden"),
        (
            "Tidewatch (2008) - S03E05 - Seasick at Sea (1080p BluRay x265 Kelp).mkv",
            "Tidewatch",
        ),
        ("Sprout Squad! S01E05-E06.mkv", "Sprout Squad!"),
        (
            "Castaway.Isle.S49E08.1080p.WEB.h264-EDNA.mkv",
            "Castaway Isle",
        ),
        (
            "Castaway Isle S49E12 1080p WEB-DL-[Ferryman1980] mkv.mkv",
            "Castaway Isle",
        ),
        (
            "The.Night.Ferry.S01E02.1080p.10bit.WEBRip.6CH.x265.HEVC-PZA.mkv",
            "The Night Ferry",
        ),
    ] {
        assert_eq!(parse(name).title.as_deref(), Some(title), "{name}");
    }
}

/// A scene sample keeps its episode's numbers; the library scan leaves samples out.
#[test]
fn scene_samples() {
    check(&[ep(
        "the.night.ferry.s02e08.1080p.web.h264-dorado.sample.mkv",
        2,
        &[8],
    )
    .group("dorado")]);
}

/// "Special" as a word of an episode title doesn't make a numbered episode a special.
#[test]
fn special_in_an_episode_title() {
    check(&[
        ep("Kettle Hill - S02E14 - Special Delivery Kite.mkv", 2, &[14]),
        ep("Brass Harbor - S01E06 - A Very Special Voyage.mp4", 1, &[6]),
        ep(
            "Storm Riders Zeta - S04E16 - Kai's Special Move.mkv",
            4,
            &[16],
        ),
        ep(
            "Storm Riders Zeta - S03E07 - Pip vs Frost! Bet it All on the Special Wave.mkv",
            3,
            &[7],
        ),
    ]);
}

/// An encoder's settings in a film's release tags (`crf19 4MAX S88`) are not a season.
#[test]
fn encoder_settings_in_a_film_name() {
    check(&[
        film(
            "Lantern Keepers and the Hollow Crown (2009)(1080p 4KBDRip DV+HDR10 x265 crf19 4MAX S88 E-AC3 5.1)[cHorsy].mkv",
        )
        .group("4MAX cHorsy"),
        film(
            "Lantern Keepers and the Stone (2001)[Theatrical Cut](1080p 4KBDRip DV+HDR10 HYBRID x265 crf19 4MAX S88 E-AC3 5.1)[cHorsy].mkv",
        )
        .group("4MAX cHorsy"),
        film(
            "Lantern Keepers and the Last Light Part 1 (2010)(1080p 4KBDRip DV+HDR10 HYBRID x265 crf19 4MAX S87 E-AC3 5.1)[cHorsy].mkv",
        )
        .part(1)
        .group("4MAX cHorsy"),
    ]);
}

/// `Title - Company - Year` names a recorded stage show, not episode 2023.
#[test]
fn a_year_after_a_dash() {
    check(&[
        film("Saltmarsh Choir - Riverside Players - 2023.mkv"),
        film("The Giant Turnip - Riverside Players - 2023.mkv"),
    ]);
}

/// `S01E29,E35`: two segments listed with a comma, in any order.
#[test]
fn comma_separated_episodes() {
    check(&[
        ep("Sprout Squad! S01E29,E35.mkv", 1, &[29, 35]),
        ep("Sprout Squad! S01E42,E40.mkv", 1, &[42, 40]),
    ]);
}

/// A Japanese broadcast capture with two episodes in one file.
#[test]
fn broadcast_capture_with_two_episodes() {
    check(&[Row {
        name: "[capsgrp] Pocket Critters (2023) - 003-004 (TVK 1440x1080 MPEG2 AAC).ts",
        kind: Kind::Regular,
        season: None,
        episodes: &[3, 4],
        part: None,
        group: Some("capsgrp"),
    }]);
}

/// The last word of an episode title is not a release group.
#[test]
#[ignore = "known words at the end of an episode title parse as a release group"]
fn episode_title_words_are_not_groups() {
    check(&[
        ep(
            "The Quiet Coast - S04E04 - Spanish for Beginners.mkv",
            4,
            &[4],
        ),
        ep("Brass Harbor - S01E06 - Cat Flaps.mp4", 1, &[6]),
        ep("Brass Harbor - S02E09 - Multi-Fins.mkv", 2, &[9]),
        ep("Kettle Hill - S10E01 - Pan(cake) Reform.mkv", 10, &[1]),
        ep(
            "Brass Harbor - S08E06 - 1958 - A Harbor Odyssey.mkv",
            8,
            &[6],
        ),
        ep("The Quiet Coast - S15E08 - 1980s Week.mkv", 15, &[8]),
        ep(
            "Sprout Squad! - S04E05-E07 - Kite Knight & Cat Flaps.mkv",
            4,
            &[5, 6, 7],
        ),
    ]);
}

/// A fansub release of two episodes in one file.
#[test]
fn fansub_double_episode() {
    let got = parse("[Grp] Tidewatch - 01-02 (1080p) [0A1B2C3D].mkv");
    assert_eq!(got.episodes, [1, 2]);
    assert_eq!(got.kind, Kind::Regular);
}
