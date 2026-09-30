//! Command-line options. They parse the way Go's `flag` package does, so the
//! forms the Go version accepted (`-dir x`, `--dir x`, `-dir=x`, `-r`) all
//! keep working.

use std::path::PathBuf;

/// What the user asked for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    /// The folder to scan.
    pub dir: PathBuf,
    /// Scan subfolders too.
    pub recursive: bool,
    /// An mpv profile to use, skipping the profile screen.
    pub profile: Option<String>,
    /// Turn mpv's deband filter off.
    pub no_deband: bool,
    /// An mpv `--hwdec` value.
    pub hwdec: Option<String>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            dir: PathBuf::from("."),
            recursive: false,
            profile: None,
            no_deband: false,
            hwdec: None,
        }
    }
}

/// The result of parsing the command line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Parsed {
    /// Run with these options.
    Run(Options),
    /// Print the usage text and exit.
    Help,
}

/// The usage text.
pub const USAGE: &str = "\
Usage: mpv-launcher [options]

  -dir <folder>     folder to scan (default: the current folder)
  -r                scan subfolders too
  -profile <name>   mpv profile to use, such as anime or music
  -nodeband         turn mpv's deband filter off
  -hwdec <mode>     mpv hardware decoding mode, such as vaapi-copy or no
  -h, -help         show this text

Each option may start with one dash or two.";

/// Parses the arguments after the program name.
///
/// # Errors
///
/// Returns a message for an unknown option, a missing value, an invalid
/// true or false value, or a stray argument.
pub fn parse<I: IntoIterator<Item = String>>(args: I) -> Result<Parsed, String> {
    let mut options = Options::default();
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        let Some(flag) = arg.strip_prefix("--").or_else(|| arg.strip_prefix('-')) else {
            return Err(format!("unexpected argument: {arg}"));
        };
        let (name, inline) = match flag.split_once('=') {
            Some((name, value)) => (name, Some(value.to_owned())),
            None => (flag, None),
        };
        let mut value = |name: &str| {
            inline
                .clone()
                .or_else(|| args.next())
                .ok_or_else(|| format!("flag needs a value: -{name}"))
        };
        match name {
            "h" | "help" => return Ok(Parsed::Help),
            "dir" => options.dir = PathBuf::from(value(name)?),
            "profile" => options.profile = Some(value(name)?),
            "hwdec" => options.hwdec = Some(value(name)?),
            "r" => options.recursive = boolean(name, inline.as_deref())?,
            "nodeband" => options.no_deband = boolean(name, inline.as_deref())?,
            _ => return Err(format!("flag provided but not defined: -{name}")),
        }
    }
    Ok(Parsed::Run(options))
}

fn boolean(name: &str, value: Option<&str>) -> Result<bool, String> {
    match value {
        None | Some("true" | "1") => Ok(true),
        Some("false" | "0") => Ok(false),
        Some(other) => Err(format!("invalid boolean value {other:?} for -{name}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(args: &[&str]) -> Options {
        match parse(args.iter().map(ToString::to_string)) {
            Ok(Parsed::Run(options)) => options,
            other => panic!("expected options, got {other:?}"),
        }
    }

    #[test]
    fn defaults_to_the_current_folder() {
        assert_eq!(run(&[]), Options::default());
    }

    #[test]
    fn accepts_go_style_single_dash_flags() {
        let options = run(&["-dir", "/tmp/show", "-r", "-profile", "anime", "-nodeband"]);
        assert_eq!(options.dir, PathBuf::from("/tmp/show"));
        assert!(options.recursive && options.no_deband);
        assert_eq!(options.profile.as_deref(), Some("anime"));
    }

    #[test]
    fn accepts_double_dashes_and_equals_signs() {
        let options = run(&["--dir=/tmp/show", "--hwdec", "no", "-r=false"]);
        assert_eq!(options.dir, PathBuf::from("/tmp/show"));
        assert_eq!(options.hwdec.as_deref(), Some("no"));
        assert!(!options.recursive);
    }

    #[test]
    fn help_and_errors() {
        assert_eq!(parse(["-h".to_owned()]), Ok(Parsed::Help));
        assert!(parse(["-dir".to_owned()]).is_err());
        assert!(parse(["-x".to_owned()]).is_err());
        assert!(parse(["folder".to_owned()]).is_err());
        assert!(parse(["-r=maybe".to_owned()]).is_err());
    }
}
