# mpv-launcher

Plays a folder of episodes in mpv. It puts the files in episode order, reads
each file's audio and subtitle tracks, and picks tracks for every file from a
ranked list of language pairings. A season where some files have Japanese
audio with English subtitles and others only have an English dub still plays
straight through, with the best pairing each file has.

```console
$ mpv-launcher -dir ~/Videos/show
Scanning directory: /home/me/Videos/show
Found 12 video files
Analyzing audio and subtitle tracks...
```

Three screens follow in a terminal. The first picks a monitor on Hyprland, or
a normal window. The second picks an mpv profile (none, `anime` or `music`)
unless `-profile` already chose one. The third puts the pairings in order:
`j` and `k` move the cursor, `J` and `K` move the pairing under it, Enter
confirms and `q` quits. The default order is:

1. Japanese + Eng Subtitles
2. English - No Subtitles
3. Japanese - No Subtitles
4. English + Eng Subtitles

A summary of which pairing each file got comes next, then mpv starts with the
whole folder as one playlist. Each file carries its own `--aid` and `--sid`,
so a change of pairing between files takes effect when that file starts.

When stdin or stdout isn't a terminal, the screens are skipped and the
defaults apply, so a script or a keybinding can run it too.

## Options

```text
-dir <folder>     folder to scan (default: the current folder)
-r                scan subfolders too
-profile <name>   mpv profile to use, such as anime or music
-nodeband         turn mpv's deband filter off
-hwdec <mode>     mpv hardware decoding mode, such as vaapi-copy or no
```

Options take one dash or two, and a value may follow `=`, so the forms the
earlier Go version accepted still work. A playlist holds at most 100 files.

mpv listens for IPC commands on `$XDG_RUNTIME_DIR/mpv-launcher.sock`.

## Installing

mpv comes from `PATH`, so your own build and `mpv.conf` apply. Reading tracks
needs GStreamer with its base and good plugins.

With Nix, the flake at the repository root has the package:

```sh
nix run github:benashby/bingekit#mpv-launcher -- -dir ~/Videos/show
```

The package sets GStreamer's plugin path itself. Without Nix, build it with
Cargo; the GStreamer development files have to be installed:

```sh
cargo install --locked --path mpv-launcher
```

## How it's put together

Episode order comes from [bingekit-episode](../episode), and scanning, probing
and track choice from [bingekit-library](../library). This crate holds the
command line, the screens (drawn with ratatui), the Hyprland monitor list and
mpv's arguments.

## Licence

0BSD. See [LICENSE](LICENSE).
