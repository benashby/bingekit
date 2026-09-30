//! The arguments mpv is started with.

use std::ffi::OsString;
use std::path::PathBuf;

use bingekit_library::Choice;

/// Options that apply to the whole playlist.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PlayOptions {
    /// An mpv profile, such as `anime`.
    pub profile: Option<String>,
    /// Turn the deband filter off.
    pub no_deband: bool,
    /// An mpv `--hwdec` value.
    pub hwdec: Option<String>,
    /// Where mpv listens for IPC commands.
    pub ipc_socket: Option<PathBuf>,
    /// A Hyprland output to play full screen on.
    pub screen: Option<String>,
}

/// One file in the playlist.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Entry {
    /// A file whose tracks were read, with the tracks chosen for it.
    Chosen(Choice),
    /// A file whose tracks could not be read, such as one that isn't
    /// Matroska. mpv picks its tracks from its own settings.
    Unread(PathBuf),
}

/// Builds mpv's arguments. Each file sits in its own `--{ … --}` block, so
/// its `--aid` and `--sid` apply to that file only.
#[must_use]
pub fn args(entries: &[Entry], options: &PlayOptions) -> Vec<OsString> {
    let mut args: Vec<OsString> = Vec::new();
    if let Some(profile) = &options.profile {
        args.push(format!("--profile={profile}").into());
    }
    if options.no_deband {
        args.push("--deband=no".into());
    }
    if let Some(hwdec) = &options.hwdec {
        args.push(format!("--hwdec={hwdec}").into());
    }
    if let Some(socket) = &options.ipc_socket {
        let mut arg = OsString::from("--input-ipc-server=");
        arg.push(socket);
        args.push(arg);
    }
    if let Some(screen) = &options.screen {
        args.push("--fs".into());
        args.push(format!("--fs-screen-name={screen}").into());
        args.push("--keepaspect=yes".into());
    }
    for entry in entries {
        args.push("--{".into());
        match entry {
            Entry::Chosen(choice) => {
                if let Some(audio) = choice.audio {
                    args.push(format!("--aid={audio}").into());
                }
                match choice.subtitles {
                    Some(sub) => args.push(format!("--sid={sub}").into()),
                    None => args.push("--sid=no".into()),
                }
                args.push(choice.path.clone().into());
            }
            Entry::Unread(path) => args.push(path.clone().into()),
        }
        args.push("--}".into());
    }
    args
}

#[cfg(test)]
mod tests {
    use super::*;

    fn choices() -> Vec<Entry> {
        vec![
            Entry::Chosen(Choice {
                path: "ep01.mkv".into(),
                audio: Some(2),
                subtitles: Some(1),
                pairing: Some(0),
            }),
            Entry::Chosen(Choice {
                path: "ep02.mkv".into(),
                audio: Some(1),
                subtitles: None,
                pairing: Some(1),
            }),
        ]
    }

    fn strings(args: &[OsString]) -> Vec<String> {
        args.iter()
            .map(|a| a.to_string_lossy().into_owned())
            .collect()
    }

    #[test]
    fn each_file_gets_its_own_block_with_its_tracks() {
        let args = strings(&args(&choices(), &PlayOptions::default()));
        assert_eq!(
            args,
            [
                "--{", "--aid=2", "--sid=1", "ep01.mkv", "--}", "--{", "--aid=1", "--sid=no",
                "ep02.mkv", "--}"
            ]
        );
    }

    #[test]
    fn a_normal_window_has_no_fullscreen_flags() {
        let args = strings(&args(&choices(), &PlayOptions::default()));
        assert!(!args.iter().any(|a| a == "--fs" || a == "--keepaspect=yes"));
    }

    #[test]
    fn a_chosen_screen_plays_full_screen_there() {
        let options = PlayOptions {
            screen: Some("HDMI-A-2".into()),
            ..PlayOptions::default()
        };
        let args = strings(&args(&choices(), &options));
        assert_eq!(
            args[..3],
            ["--fs", "--fs-screen-name=HDMI-A-2", "--keepaspect=yes"]
        );
    }

    #[test]
    fn global_options_come_before_the_files() {
        let options = PlayOptions {
            profile: Some("anime".into()),
            no_deband: true,
            hwdec: Some("vaapi-copy".into()),
            ipc_socket: Some("/run/user/1000/mpv-launcher.sock".into()),
            screen: None,
        };
        let args = strings(&args(&choices(), &options));
        assert_eq!(
            args[..4],
            [
                "--profile=anime",
                "--deband=no",
                "--hwdec=vaapi-copy",
                "--input-ipc-server=/run/user/1000/mpv-launcher.sock"
            ]
        );
        assert_eq!(args[4], "--{");
    }

    #[test]
    fn a_file_without_audio_gets_no_aid() {
        let silent = Entry::Chosen(Choice {
            path: "x.mkv".into(),
            audio: None,
            subtitles: None,
            pairing: None,
        });
        let args = strings(&args(&[silent], &PlayOptions::default()));
        assert_eq!(args, ["--{", "--sid=no", "x.mkv", "--}"]);
    }

    #[test]
    fn an_unread_file_leaves_its_tracks_to_mpv() {
        let unread = Entry::Unread("clip.mp4".into());
        let args = strings(&args(&[unread], &PlayOptions::default()));
        assert_eq!(args, ["--{", "clip.mp4", "--}"]);
    }
}
