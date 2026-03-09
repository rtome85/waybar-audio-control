# waybar-audio-control

A GTK4 Wayland audio control popup for [waybar](https://github.com/Alexays/Waybar), built with [gtk4-layer-shell](https://github.com/wmww/gtk4-layer-shell) and [libpulse](https://www.freedesktop.org/wiki/Software/PulseAudio/).

Displays a popup in the top-right corner of the screen with per-application volume sliders, playback device selection, and input device selection. Dismisses when clicking outside the popup.

## Features

- Per-application volume control (sink inputs)
- Playback device selection (sinks)
- Input device selection (sources)
- Positioned at top-right corner via `gtk4-layer-shell`
- Dismisses on click outside the popup
- Auto-refreshes audio state every 2 seconds

## Requirements

- Wayland compositor with [wlr-layer-shell](https://wayland.app/protocols/wlr-layer-shell-unstable-v1) support (Hyprland, Sway, etc.)
- PulseAudio or PipeWire with PulseAudio compatibility layer
- GTK4
- gtk4-layer-shell

## Building

```bash
cargo build --release
```

The binary will be at `target/release/audio-control`.

## Waybar Integration

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

## Architecture

```
main.rs   — Application entry point, backdrop window for click-outside dismissal
ui.rs     — GTK4 UI layout, layer shell setup, audio controls rendering
audio.rs  — PulseAudio interface (sink inputs, sinks, sources, volume control)
```

**Dismissal mechanism:** A fullscreen transparent backdrop window sits at `Layer::Top`. The popup itself is at `Layer::Overlay` (above everything). Clicking outside the popup hits the backdrop, which closes both windows.

## License

MIT
