//! Properties that must hold for any input.
use bingekit_episode::{Kind, parse};
use proptest::prelude::*;

proptest! {
    /// Arbitrary text never panics, and the file name is kept verbatim.
    #[test]
    fn never_panics(name in "\\PC{0,120}") {
        let ep = parse(&name);
        prop_assert_eq!(ep.file_name, name);
    }

    /// Release noise around an anime episode number never changes the number.
    #[test]
    fn anime_noise_keeps_episode(
        n in 1u32..2000,
        group in "[A-Za-z]{2,10}",
        res in prop_oneof!["1920x1080", "1280x720", "3840x2160", "1080p"],
        crc in "[0-9A-F]{8}",
    ) {
        let name = format!("[{group}]_Some_Show_-_{n:02}_({res}_H264_10bit)_[{crc}].mkv");
        let ep = parse(&name);
        prop_assert_eq!(ep.kind, Kind::Regular);
        prop_assert_eq!(ep.episodes, vec![n]);
    }
}
