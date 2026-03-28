# waybar-audio-control

A GTK4 Wayland audio control popup for [waybar](https://github.com/Alexays/Waybar), built with [gtk4-layer-shell](https://github.com/wmww/gtk4-layer-shell) and [libpulse](https://www.freedesktop.org/wiki/Software/PulseAudio/).

Displays a popup in the top-right corner of the screen with a media player carousel, per-application volume sliders, playback device selection, and input device selection. Dismisses when clicking outside the popup.

## Features

- **Now Playing carousel** — shows active MPRIS media players with track, artist, and playback controls (prev/play/next); swipe between players with dot navigation
- **Per-application volume control** — streams are grouped by application name so each app gets one slider regardless of how many PulseAudio sink inputs it opens
- **Playback device selection** — lists all sinks, marks the current default, click to switch
- **Input device selection** — lists all sources (monitors excluded), marks the current default, click to switch
- **Persistent background process** — the process stays alive and `SIGUSR1` toggles the window, so subsequent clicks are instant
- **Dynamic theming** — reads accent, background, and surface colors from `~/.config/omarchy/current/theme/`
- **Positioned at top-right corner** via `gtk4-layer-shell`
- **Dismisses on click outside** the popup
- **Auto-refreshes** audio and media state every 2 seconds

## Requirements

- Wayland compositor with [wlr-layer-shell](https://wayland.app/protocols/wlr-layer-shell-unstable-v1) support (Hyprland, Sway, etc.)
- PulseAudio or PipeWire with PulseAudio compatibility layer
- GTK4
- gtk4-layer-shell
- A [Nerd Font](https://www.nerdfonts.com/) for application icons and media control glyphs

## Building

```bash
cargo build --release
```

The binary will be at `target/release/audio-control`.

## Waybar Integration

The process persists in the background after first launch and uses a PID file at `/tmp/audio-control.pid` for IPC. Subsequent waybar clicks send `SIGUSR1` to the running process to toggle visibility instead of spawning a new instance.

Add a custom module to your waybar config (`~/.config/waybar/config`):

```json
"custom/audio": {
    "format": "󰕾",
    "on-click": "/path/to/audio-control",
    "tooltip": false
}
```

Add it to your bar's modules:

```json
"modules-right": ["custom/audio", "clock", ...]
```

## Dependencies

| Crate | Version | Purpose |
|---|---|---|
| `gtk4` | 0.9 | UI framework |
| `gtk4-layer-shell` | 0.4 | Wayland layer shell integration |
| `gdk4` | 0.9 | GDK bindings |
| `glib` | 0.20 | GLib utilities |
| `libpulse-binding` | 2.28 | PulseAudio interface |
| `mpris` | 2 | MPRIS D-Bus media player control |
| `libc` | 0.2 | POSIX signal handling |

## Architecture

```
main.rs   — Entry point; PID file IPC; SIGUSR1 toggle; backdrop + popup window setup
ui.rs     — GTK4 UI layout, layer shell config, CSS theming, all section renderers
audio.rs  — PulseAudio interface (sink inputs, sinks, sources, volume control)
media.rs  — MPRIS integration (player discovery, playback control)
```

**Dismissal mechanism:** A fullscreen transparent backdrop window sits at `Layer::Top`. The popup itself is at `Layer::Overlay` (above everything). Clicking outside the popup hits the backdrop, which hides both windows. `Alt+F4` / compositor close requests are intercepted and treated as hide instead of quit.

**Grouping:** Multiple PulseAudio sink inputs from the same application (e.g. a browser with several audio tabs) are collapsed into a single volume slider. Moving the slider sets volume on all of that app's underlying sink inputs simultaneously.

**Media carousel:** Each active MPRIS player (playing or paused — stopped players are excluded) gets a card showing the app icon, track title, artist, and transport controls. Cards are navigated with dot indicators at the bottom of the section.

## License

MIT
