//! Video files in viewing order.

use std::collections::HashMap;
use std::collections::VecDeque;
use std::path::PathBuf;

/// Puts `paths` in viewing order. Episodes are parsed from file names alone,
/// because the release-name conventions live in the name and the folders
/// above it would only confuse the parser. Two files with the same name in
/// different subfolders keep their scan order.
#[must_use]
pub fn in_viewing_order(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut by_name: HashMap<String, VecDeque<PathBuf>> = HashMap::new();
    let mut names = Vec::with_capacity(paths.len());
    for path in paths {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        names.push(name.clone());
        by_name.entry(name).or_default().push_back(path);
    }
    bingekit_episode::parse_folder(&names)
        .into_iter()
        .filter_map(|ep| by_name.get_mut(&ep.file_name)?.pop_front())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orders_by_episode_with_specials_last() {
        let paths = vec![
            PathBuf::from("/tv/[Grp] Show - S01 [1080p].mkv"),
            PathBuf::from("/tv/[Grp] Show - 10 [1080p].mkv"),
            PathBuf::from("/tv/[Grp] Show - 02 [1080p].mkv"),
        ];
        let order: Vec<_> = in_viewing_order(paths)
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        assert_eq!(
            order,
            [
                "[Grp] Show - 02 [1080p].mkv",
                "[Grp] Show - 10 [1080p].mkv",
                "[Grp] Show - S01 [1080p].mkv"
            ]
        );
    }

    #[test]
    fn keeps_every_file_when_names_repeat_across_folders() {
        let paths = vec![
            PathBuf::from("/tv/s1/Show.S01E01.mkv"),
            PathBuf::from("/tv/s1-copy/Show.S01E01.mkv"),
            PathBuf::from("/tv/s1/Show.S01E02.mkv"),
        ];
        let order = in_viewing_order(paths);
        assert_eq!(order.len(), 3);
        assert_eq!(order[0], PathBuf::from("/tv/s1/Show.S01E01.mkv"));
        assert_eq!(order[1], PathBuf::from("/tv/s1-copy/Show.S01E01.mkv"));
    }
}
