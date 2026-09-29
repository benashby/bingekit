//! Golden tests: every name in `testdata/episode-names.txt` parses to a reviewed snapshot.
//! Add a line, run `cargo insta review`, accept or fix.

#[test]
fn episode_names() {
    let list = include_str!("../testdata/episode-names.txt");
    for name in list
        .lines()
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
    {
        let ep = bingekit_episode::parse(name);
        insta::with_settings!({ description => name, snapshot_suffix => slug(name) }, {
            insta::assert_debug_snapshot!(ep);
        });
    }
}

fn slug(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .take(60)
        .collect()
}
