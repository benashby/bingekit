//! Episode information from media filenames, and a viewing order for a folder of them.
//!
//! General release names are parsed by [`hunch`], a Rust port of guessit.
//! Anime fansub releases (`[Group] Title - 05v2 (1080p) [CRC].mkv`) follow
//! conventions of their own, and a small layer here handles them: the number after the
//! last ` - ` is the episode, `S01` means special 1 rather than season 1, `C01`,
//! `NCOP` and `NCED` are creditless extras, and `v2` is a re-release.
//!
//! ```
//! let ep = bingekit_episode::parse("[Grp]_Paper_Lantern_Club_-_05v2_(1920x1080_H264_10bit)_[1A2B3C4D].mkv");
//! assert_eq!(ep.episodes, vec![5]);
//! assert_eq!(ep.version, Some(2));
//! assert_eq!(ep.kind, bingekit_episode::Kind::Regular);
//! ```

use std::cmp::Ordering;
use std::sync::LazyLock;

use hunch::{HunchResult, Pipeline, Property};
use regex::Regex;

mod play;
pub use play::{next_after, play_order};

// Runs the README example as a doctest, so the README cannot drift from the code.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
pub struct ReadmeDoctests;

/// What a file is, for ordering and display.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Kind {
    /// A numbered episode of the main run.
    Regular,
    /// A special, OVA or OAD: part of the story, but outside the main numbering.
    Special,
    /// Creditless openings and endings, previews, bonus material.
    Extra,
    /// A single film.
    Movie,
    /// Nothing recognisable.
    Unknown,
}

/// Parsed information about one media file.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Episode {
    /// The file name this was parsed from.
    pub file_name: String,
    /// The show or film title, if one was found.
    pub title: Option<String>,
    /// Season number, when the name carries one (`S02E05`, `2x05`).
    pub season: Option<u32>,
    /// Episode numbers: usually one, several for multi-episode files, none if absent.
    pub episodes: Vec<u32>,
    /// Classification, which drives ordering.
    pub kind: Kind,
    /// Release version (`v2`), when marked.
    pub version: Option<u32>,
    /// Part of a split episode (`Part 2`).
    pub part: Option<u32>,
    /// The release group (`[Grp]`, `-RLS`).
    pub release_group: Option<String>,
}

impl Episode {
    /// The first episode number, if any.
    #[must_use]
    pub fn episode(&self) -> Option<u32> {
        self.episodes.first().copied()
    }

    fn kind_rank(&self) -> u8 {
        match self.kind {
            Kind::Regular => 0,
            Kind::Special => 1,
            Kind::Extra => 2,
            Kind::Movie => 3,
            Kind::Unknown => 4,
        }
    }

    /// Viewing order: regular episodes by season, episode and part, then specials, extras,
    /// films and unrecognised files. Ties fall back to the file name, so the order is total
    /// and stable across runs.
    #[must_use]
    pub fn viewing_order(&self, other: &Self) -> Ordering {
        self.kind_rank()
            .cmp(&other.kind_rank())
            .then(self.season.unwrap_or(1).cmp(&other.season.unwrap_or(1)))
            .then(
                self.episode()
                    .unwrap_or(u32::MAX)
                    .cmp(&other.episode().unwrap_or(u32::MAX)),
            )
            .then(self.part.unwrap_or(0).cmp(&other.part.unwrap_or(0)))
            .then_with(|| self.file_name.cmp(&other.file_name))
    }
}

/// Parse one file name on its own. Prefer [`parse_folder`] when the file's siblings are
/// known: cross-file context settles ambiguous titles.
#[must_use]
pub fn parse(file_name: &str) -> Episode {
    let pipeline = Pipeline::new();
    parse_with(&pipeline, file_name, &[])
}

/// Parse every file name in a folder, using the others as context for each one, and
/// return them in viewing order.
#[must_use]
pub fn parse_folder<S: AsRef<str>>(file_names: &[S]) -> Vec<Episode> {
    let pipeline = Pipeline::new();
    let names: Vec<&str> = file_names.iter().map(AsRef::as_ref).collect();
    // A name that reads as a dated film on its own stays a film, and isn't context
    // for the others: the numbers that differ between film titles are sequels, not
    // episodes.
    let films: Vec<bool> = names
        .iter()
        .map(|n| parse_anime_release(n).is_none() && is_dated_film(&pipeline.run(n)))
        .collect();
    let mut parsed: Vec<Episode> = names
        .iter()
        .enumerate()
        .map(|(i, name)| {
            if films[i] {
                return parse_with(&pipeline, name, &[]);
            }
            let siblings: Vec<&str> = names
                .iter()
                .enumerate()
                .filter_map(|(j, n)| (j != i && !films[j]).then_some(*n))
                .collect();
            parse_with(&pipeline, name, &siblings)
        })
        .collect();
    parsed.sort_by(Episode::viewing_order);
    parsed
}

fn parse_with(pipeline: &Pipeline, file_name: &str, siblings: &[&str]) -> Episode {
    if let Some(anime) = parse_anime_release(file_name) {
        return anime;
    }
    let r = if siblings.is_empty() {
        pipeline.run(file_name)
    } else {
        pipeline.run_with_context(file_name, siblings)
    };
    // A dated film's other numbers are not a season or episodes: `S88` in an
    // encoder's settings, or the year itself after a dash.
    let film = is_dated_film(&r);
    let season = r
        .season()
        .and_then(|s| u32::try_from(s).ok())
        .filter(|_| !film);
    let mut episodes: Vec<u32> = r
        .all(Property::Episode)
        .iter()
        .filter_map(|e| e.parse().ok())
        .filter(|_| !film)
        .collect();
    // hunch reads only the first of `S01E29,E35`.
    if let Some(listed) = comma_listed_episodes(file_name)
        && episodes.first() == listed.first()
    {
        episodes = listed;
    }
    let details = r.episode_details().unwrap_or_default().to_ascii_lowercase();
    // hunch also finds "Special" in an episode title. A season and episode number
    // already place the file in the main run, where season 0 holds the specials.
    let numbered = season.is_some_and(|s| s > 0) && !episodes.is_empty();
    let kind = if season == Some(0)
        || (!numbered && (details.contains("special") || details.contains("ova")))
    {
        Kind::Special
    } else if r.is_extra() {
        Kind::Extra
    } else if !episodes.is_empty() || (r.is_episode() && !film) {
        Kind::Regular
    } else if r.is_movie() || film {
        Kind::Movie
    } else {
        Kind::Unknown
    };
    Episode {
        file_name: file_name.to_owned(),
        title: r.title().map(str::to_owned),
        season,
        episodes,
        kind,
        version: r.first(Property::Version).and_then(|v| v.parse().ok()),
        part: r.part().and_then(|p| u32::try_from(p).ok()),
        release_group: r.release_group().map(str::to_owned),
    }
}

/// A name with a year and no episode number other than that year, and no air date:
/// a film.
fn is_dated_film(r: &HunchResult) -> bool {
    let Some(year) = r.year() else {
        return false;
    };
    let episodes = r.all(Property::Episode);
    r.date().is_none() && (episodes.is_empty() || episodes == [year.to_string()])
}

/// `S01E29,E35`: episodes listed with commas, in the order written.
static COMMA_LISTED: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\bS\d{1,3}E(\d{1,4})((?:,E\d{1,4})+)\b").expect("static regex")
});

fn comma_listed_episodes(file_name: &str) -> Option<Vec<u32>> {
    let caps = COMMA_LISTED.captures(file_name)?;
    let rest = caps[2].split(',').skip(1).map(|e| &e[1..]);
    std::iter::once(&caps[1])
        .chain(rest)
        .map(|n| n.parse().ok())
        .collect()
}

/// `[Group] Title - <tag><n>[v<ver>][-<last>] …`, with spaces or underscores. The tag
/// decides the kind: none is a regular episode, `S`/`SP`/`OVA`/`OAD` a special, `C`/
/// `NC`/`NCOP`/`NCED`/`OP`/`ED`/`PV` an extra. `-<last>` makes it a run of episodes.
static ANIME: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?ix)
        ^\[(?P<group>[^\]]+)\][\s_]*
        (?P<title>.+?)
        [\s_]-[\s_]
        (?P<tag>NCOP|NCED|OVA|OAD|SP|NC|OP|ED|PV|S|C)?
        (?P<num>\d{1,4})
        (?:v(?P<ver>\d{1,2}))?
        (?:-(?P<last>\d{1,4}))?
        (?:[\s_.(\[]|$)",
    )
    .expect("static regex")
});

fn parse_anime_release(file_name: &str) -> Option<Episode> {
    let stem = file_name
        .rsplit_once('.')
        .filter(|(_, ext)| ext.len() <= 4 && ext.chars().all(char::is_alphanumeric))
        .map_or(file_name, |(stem, _)| stem);
    let caps = ANIME.captures(stem)?;
    let kind = match caps.name("tag").map(|t| t.as_str().to_ascii_uppercase()) {
        None => Kind::Regular,
        Some(t) if matches!(t.as_str(), "S" | "SP" | "OVA" | "OAD") => Kind::Special,
        Some(_) => Kind::Extra,
    };
    let title = caps["title"].replace('_', " ").trim().to_owned();
    let first: u32 = caps["num"].parse().ok()?;
    let last = caps
        .name("last")
        .and_then(|l| l.as_str().parse().ok())
        .filter(|&l| l > first)
        .unwrap_or(first);
    Some(Episode {
        file_name: file_name.to_owned(),
        title: (!title.is_empty()).then_some(title),
        season: None,
        episodes: (first..=last).collect(),
        kind,
        version: caps.name("ver").and_then(|v| v.as_str().parse().ok()),
        part: None,
        release_group: Some(caps["group"].to_owned()),
    })
}
