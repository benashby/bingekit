//! What plays, and what plays next, from a folder that may hold the same episode
//! more than once.
//!
//! A folder often keeps two releases of a show side by side: two fansub groups, a
//! 1080p and a 720p copy, a `v2` beside the file it replaced, or a TV season whose
//! episodes came from different groups. [`parse_folder`](crate::parse_folder) lists
//! them all, which is right for showing the folder. Playing it should give one file per
//! episode, and that file should come from the same release as the one the viewer
//! started with, so the audio tracks, subtitles and chapters stay the same from one
//! episode to the next.

use std::collections::HashSet;

use crate::{Episode, Kind, parse_folder};

/// The files to play from a folder, in viewing order, one per episode.
///
/// `file_names` is every file in the folder, as for [`parse_folder`](crate::parse_folder).
/// `start` names the file playback starts from; the files kept are the ones that look
/// most like it. With no `start`, the first file in viewing order sets the release.
///
/// Where an episode has several files, the one kept is, in order of preference:
///
/// 1. `start` itself;
/// 2. a file from `start`'s release group;
/// 3. the file whose name, with its numbers set aside, shares the most words with
///    `start`'s (resolution, source, codec and the like);
/// 4. the highest version (`v2` over the original);
/// 5. the first in viewing order.
///
/// An episode that only another release has stays in, so a season whose episodes came
/// from different groups plays through. A file covering several episodes stands in for
/// the single files of the same episodes when it wins. Extras, films and files with no
/// episode number are never merged, since nothing says which of them are copies.
#[must_use]
pub fn play_order<S: AsRef<str>>(file_names: &[S], start: Option<&str>) -> Vec<Episode> {
    let episodes = parse_folder(file_names);
    let Some(reference) = start
        .and_then(|s| episodes.iter().find(|e| e.file_name == s))
        .or_else(|| episodes.first())
        .cloned()
    else {
        return Vec::new();
    };
    let reference_words = words(&reference.file_name);
    let score = |e: &Episode| {
        (
            e.file_name == reference.file_name,
            reference.release_group.is_some() && e.release_group == reference.release_group,
            shared(&reference_words, &words(&e.file_name)),
            e.version.unwrap_or(1),
        )
    };

    // Files that are the same episode, in viewing order of their first file.
    let mut slots: Vec<Vec<Episode>> = Vec::new();
    for ep in episodes {
        let slot = mergeable(&ep)
            .then(|| {
                slots
                    .iter()
                    .position(|s| s.iter().any(|o| same_episode(o, &ep)))
            })
            .flatten();
        match slot {
            Some(i) => slots[i].push(ep),
            None => slots.push(vec![ep]),
        }
    }

    let mut out = Vec::new();
    for slot in slots {
        if slot.len() == 1 {
            out.extend(slot);
            continue;
        }
        let Some(best) = slot
            .iter()
            .enumerate()
            // Ties go to the earlier file, which max_by_key would not give.
            .max_by(|(i, a), (j, b)| score(a).cmp(&score(b)).then(j.cmp(i)))
            .map(|(_, e)| e.clone())
        else {
            continue;
        };
        // The winner's release may split the slot into several files (singles where
        // another release has one double episode). Keep each of its files that covers
        // an episode the winner doesn't.
        let mut covered: HashSet<u32> = best.episodes.iter().copied().collect();
        let mut kept = vec![best.clone()];
        for e in &slot {
            if e.file_name != best.file_name
                && e.release_group == best.release_group
                && e.episodes.iter().all(|n| !covered.contains(n))
            {
                covered.extend(e.episodes.iter().copied());
                kept.push(e.clone());
            }
        }
        kept.sort_by(Episode::viewing_order);
        out.extend(kept);
    }
    out
}

/// The file that plays after `current` in its folder: the next one in
/// [`play_order`] from `current`, or `None` when `current` is the last or isn't in
/// `file_names`.
#[must_use]
pub fn next_after<S: AsRef<str>>(file_names: &[S], current: &str) -> Option<Episode> {
    let order = play_order(file_names, Some(current));
    let i = order.iter().position(|e| e.file_name == current)?;
    order.into_iter().nth(i + 1)
}

/// Whether a file can stand for the same episode as another: a numbered episode or
/// special.
fn mergeable(ep: &Episode) -> bool {
    !ep.episodes.is_empty() && matches!(ep.kind, Kind::Regular | Kind::Special)
}

/// Two files of the same kind, season and part that share an episode number.
fn same_episode(a: &Episode, b: &Episode) -> bool {
    mergeable(a)
        && mergeable(b)
        && a.kind == b.kind
        && a.season.unwrap_or(1) == b.season.unwrap_or(1)
        && a.part.unwrap_or(0) == b.part.unwrap_or(0)
        && a.episodes.iter().any(|n| b.episodes.contains(n))
}

/// The words of a file name that describe its release rather than its episode:
/// lower case, without the extension, episode numbers, versions, season and episode
/// markers or CRCs.
fn words(file_name: &str) -> HashSet<String> {
    let stem = file_name
        .rsplit_once('.')
        .filter(|(_, ext)| ext.len() <= 4 && ext.chars().all(char::is_alphanumeric))
        .map_or(file_name, |(stem, _)| stem);
    stem.split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(str::to_ascii_lowercase)
        .filter(|w| !is_numbering(w))
        .collect()
}

/// The tags an episode number may carry in a release name (`S01`, `C01`, `NCOP1`,
/// `E05`).
const NUMBER_TAGS: [&str; 14] = [
    "", "s", "sp", "c", "nc", "ncop", "nced", "op", "ed", "ova", "oad", "pv", "e", "ep",
];

/// An episode number (`05`, `05v2`, `c01`, `s01v2`), a season and episode marker
/// (`s01e05`, `s01e01e02`, `2x05`) or an eight-digit CRC.
fn is_numbering(w: &str) -> bool {
    let letters = w.trim_end_matches(|c: char| c.is_ascii_digit() || c == 'v');
    let tail = &w[letters.len()..];
    let numbered = tail.starts_with(|c: char| c.is_ascii_digit()) && NUMBER_TAGS.contains(&letters);
    let crc = w.len() == 8 && w.chars().all(|c| c.is_ascii_hexdigit());
    let marker = |w: &str| {
        let mut parts = w.split(['e', 'x']);
        let first = parts.next().unwrap_or_default().trim_start_matches('s');
        !first.is_empty()
            && first.chars().all(|c| c.is_ascii_digit())
            && first.len() <= 2
            && parts.clone().count() >= 1
            && parts.all(|p| !p.is_empty() && p.len() <= 3 && p.chars().all(|c| c.is_ascii_digit()))
    };
    numbered || crc || marker(w)
}

/// How many words two names share.
fn shared(a: &HashSet<String>, b: &HashSet<String>) -> usize {
    a.intersection(b).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbering_words() {
        for w in [
            "05",
            "05v2",
            "c01",
            "s01v2",
            "s01e05",
            "s01e01e02",
            "2x05",
            "1a2b3c4d",
            "ncop1",
        ] {
            assert!(is_numbering(w), "{w} is numbering");
        }
        for w in ["1920x1080", "1080p", "h264", "10bit", "x264", "web", "flac"] {
            assert!(!is_numbering(w), "{w} describes the release");
        }
    }
}
