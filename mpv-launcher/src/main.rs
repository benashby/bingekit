//! Plays a folder of episodes in mpv: in episode order, with audio and
//! subtitle tracks picked for each file from ranked language pairings.

mod args;
mod episodes;
mod hyprland;
mod mpv;
mod ui;

use std::io::IsTerminal;
use std::path::PathBuf;
use std::process::{Command, ExitCode};
use std::time::Duration;

use bingekit_library::probe::Prober;
use bingekit_library::{ScanOptions, choose_tracks, default_pairings, scan, summary};

use args::{Options, Parsed};
use mpv::PlayOptions;

/// The most files one playlist takes.
const MAX_FILES: usize = 100;

fn main() -> ExitCode {
    match args::parse(std::env::args().skip(1)) {
        Ok(Parsed::Help) => {
            println!("{}", args::USAGE);
            ExitCode::SUCCESS
        }
        Ok(Parsed::Run(options)) => match run(&options) {
            Ok(code) => code,
            Err(message) => {
                eprintln!("Error: {message}");
                ExitCode::FAILURE
            }
        },
        Err(message) => {
            eprintln!("{message}\n\n{}", args::USAGE);
            ExitCode::from(2)
        }
    }
}

fn run(options: &Options) -> Result<ExitCode, String> {
    println!("Scanning directory: {}", options.dir.display());
    let scan_options = ScanOptions {
        recursive: options.recursive,
        follow_symlinks: false,
    };
    let mut videos = scan(&options.dir, scan_options).map_err(|e| e.to_string())?;
    if videos.is_empty() {
        println!("No video files found in directory");
        return Ok(ExitCode::FAILURE);
    }
    if videos.len() > MAX_FILES {
        println!(
            "Warning: Found {} files, limiting to {MAX_FILES}",
            videos.len()
        );
        videos.truncate(MAX_FILES);
    }
    println!("Found {} video files", videos.len());
    let videos = episodes::in_viewing_order(videos);

    println!("Analyzing audio and subtitle tracks...");
    let prober = Prober::new(Duration::from_secs(10)).map_err(|e| e.to_string())?;
    let mut files = Vec::with_capacity(videos.len());
    for path in &videos {
        match prober.probe(path) {
            Ok(file) => files.push(file),
            Err(e) => eprintln!("Skipping {}: {e}", path.display()),
        }
    }
    if files.is_empty() {
        return Err("No valid files to analyze".to_owned());
    }

    // The screens need a terminal. Without one (a script or a pipe), the
    // defaults apply: a normal window, no profile unless -profile gave one,
    // and the pairings in their default order.
    let interactive = std::io::stdin().is_terminal() && std::io::stdout().is_terminal();

    let mut play = PlayOptions {
        profile: options.profile.clone(),
        no_deband: options.no_deband,
        hwdec: options.hwdec.clone(),
        ipc_socket: std::env::var_os("XDG_RUNTIME_DIR")
            .map(|dir| PathBuf::from(dir).join("mpv-launcher.sock")),
        screen: None,
    };

    if interactive && hyprland::is_available() {
        match hyprland::monitors() {
            Ok(monitors) => {
                let mut items = vec!["Normal window".to_owned()];
                items.extend(monitors.iter().map(hyprland::Monitor::label));
                println!();
                match ui::pick("Select monitor:", items).map_err(|e| e.to_string())? {
                    None => return Ok(canceled()),
                    Some(0) => {}
                    Some(i) => play.screen = Some(monitors[i - 1].name.clone()),
                }
            }
            Err(e) => eprintln!("Warning: could not query Hyprland monitors: {e}"),
        }
    }

    if interactive && play.profile.is_none() {
        let profiles = ["None", "anime", "music"];
        println!();
        let items = profiles.iter().map(ToString::to_string).collect();
        match ui::pick("Select profile:", items).map_err(|e| e.to_string())? {
            None => return Ok(canceled()),
            Some(0) => {}
            Some(i) => play.profile = Some(profiles[i].to_owned()),
        }
    }

    let defaults = default_pairings();
    let pairings = if interactive {
        let labels = defaults.iter().map(|p| p.label.clone()).collect();
        println!();
        let prompt = "Set playback priority (j/k move, J/K reorder, Enter confirm, q quit):";
        match ui::reorder(prompt, labels).map_err(|e| e.to_string())? {
            None => return Ok(canceled()),
            Some(order) => order.into_iter().map(|i| defaults[i].clone()).collect(),
        }
    } else {
        defaults
    };

    let choices = choose_tracks(&files, &pairings);
    println!("Track selection summary:");
    for (label, count) in summary(&choices, &pairings) {
        println!("  {label}: {count} files");
    }
    println!();

    println!("Launching MPV with {} files...\n", choices.len());
    let status = Command::new("mpv")
        .args(mpv::args(&choices, &play))
        .status()
        .map_err(|e| format!("could not start mpv: {e}"))?;
    if !status.success() {
        return Err(format!("mpv exited with {status}"));
    }
    println!("\nPlayback finished");
    Ok(ExitCode::SUCCESS)
}

fn canceled() -> ExitCode {
    println!("Selection canceled");
    ExitCode::SUCCESS
}
