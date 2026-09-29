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
