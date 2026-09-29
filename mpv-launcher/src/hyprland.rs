//! The monitors Hyprland knows about, read from its control socket.

use std::env;
use std::io::{self, Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

use serde::Deserialize;

/// One monitor, as Hyprland's `monitors` command reports it.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Monitor {
    /// The output name, such as `DP-1`.
    pub name: String,
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Refresh rate in hertz.
    pub refresh_rate: f64,
    /// Whether the monitor has focus.
    #[serde(default)]
    pub focused: bool,
    /// Whether the monitor is turned off.
    #[serde(default)]
    pub disabled: bool,
}

impl Monitor {
    /// The line shown in the monitor picker.
    #[must_use]
    pub fn label(&self) -> String {
        let mut label = format!(
            "{} - {}x{} @ {:.0}Hz",
            self.name, self.width, self.height, self.refresh_rate
        );
        if self.focused {
            label.push_str(" [focused]");
        }
        label
    }
}

/// Whether this process runs inside a Hyprland session.
#[must_use]
pub fn is_available() -> bool {
    env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_some()
}

/// The enabled monitors.
///
/// # Errors
///
/// Fails when the session variables are missing, the socket cannot be used,
/// or the reply is not the expected JSON.
pub fn monitors() -> io::Result<Vec<Monitor>> {
    let signature = env::var_os("HYPRLAND_INSTANCE_SIGNATURE")
        .ok_or_else(|| io::Error::other("HYPRLAND_INSTANCE_SIGNATURE is not set"))?;
    let runtime = env::var_os("XDG_RUNTIME_DIR")
        .ok_or_else(|| io::Error::other("XDG_RUNTIME_DIR is not set"))?;
    let socket = PathBuf::from(runtime)
        .join("hypr")
        .join(signature)
        .join(".socket.sock");
    let mut stream = UnixStream::connect(socket)?;
    stream.write_all(b"-j/monitors")?;
    let mut reply = String::new();
    stream.read_to_string(&mut reply)?;
    parse(&reply).map_err(io::Error::other)
}

/// Parses the JSON reply to `monitors` and drops disabled monitors.
///
/// # Errors
///
/// Fails when the reply is not a JSON list of monitors.
pub fn parse(reply: &str) -> Result<Vec<Monitor>, serde_json::Error> {
    let all: Vec<Monitor> = serde_json::from_str(reply)?;
    Ok(all.into_iter().filter(|m| !m.disabled).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_reply_and_drops_disabled_monitors() {
        let reply = r#"[
            {"id": 0, "name": "DP-1", "width": 5120, "height": 2880, "refreshRate": 165.0,
             "focused": true, "disabled": false, "description": "a monitor"},
            {"id": 1, "name": "HDMI-A-2", "width": 1920, "height": 1080, "refreshRate": 59.94,
             "focused": false, "disabled": false},
            {"id": 2, "name": "DP-3", "width": 1920, "height": 1080, "refreshRate": 60.0,
             "focused": false, "disabled": true}
        ]"#;
        let monitors = parse(reply).unwrap();
        let labels: Vec<_> = monitors.iter().map(Monitor::label).collect();
        assert_eq!(
            labels,
            [
                "DP-1 - 5120x2880 @ 165Hz [focused]",
                "HDMI-A-2 - 1920x1080 @ 60Hz"
            ]
        );
    }

    #[test]
    fn a_bad_reply_is_an_error() {
        assert!(parse("unknown request").is_err());
    }
}
