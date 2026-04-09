use crate::audio::{AudioDevice, AudioManager, AudioStream};
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box, Button, Label, Orientation, Scale, Separator};
use gtk4 as gtk;
use gtk4_layer_shell::{Layer, LayerShell};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

fn app_icon(app_name: &str) -> &'static str {
    let lower = app_name.to_lowercase();
    if lower.contains("firefox") {
        "\u{f269}" // nf-fa-firefox
    } else if lower.contains("chrome") || lower.contains("chromium") {
        "\u{f268}" // nf-fa-chrome
    } else if lower.contains("spotify") {
        "\u{f1bc}" // nf-fa-spotify
    } else if lower.contains("vlc") {
        "\u{f04b}" // nf-fa-play
    } else if lower.contains("mpv") {
        "\u{f04b}" // nf-fa-play
    } else if lower.contains("discord") {
        "\u{f392}" // nf-fa-discord
    } else if lower.contains("steam") {
        "\u{f1b6}" // nf-fa-steam
    } else if lower.contains("telegram") {
        "\u{f2c6}" // nf-fa-telegram
    } else if lower.contains("zoom") {
        "\u{f0c0}" // nf-fa-users
    } else if lower.contains("brave") {
        "\u{f268}" // nf-fa-chrome (similar)
    } else if lower.contains("obs") {
        "\u{f03d}" // nf-fa-video_camera
    } else if lower.contains("pulse") || lower.contains("audio") {
        "\u{f028}" // nf-fa-volume_up
    } else {
        "\u{f001}" // nf-fa-music
    }
}

fn build_css() -> String {
    let home = std::env::var("HOME").unwrap_or_default();
    let theme_dir = format!("{}/.config/omarchy/current/theme", home);

    // Parse fg/bg from waybar.css — same source waybar uses.
    // We use private names (_fg, _bg) to avoid conflicts with GTK4 reserved color names.
    let fg = parse_color_from_file(&format!("{}/waybar.css", theme_dir), "foreground")
        .unwrap_or_else(|| "#cdd6f4".to_string());
    let bg = parse_color_from_file(&format!("{}/waybar.css", theme_dir), "background")
        .unwrap_or_else(|| "#1e1e2e".to_string());

    // accent is not in waybar.css; read it from colors.toml
    let accent = parse_color_from_file(&format!("{}/colors.toml", theme_dir), "accent")
        .unwrap_or_else(|| "#f5c2e7".to_string());

    format!(
        r#"
@define-color _fg {fg};
@define-color _bg {bg};
@define-color _accent {accent};
@define-color _surface alpha(@_fg, 0.08);
@define-color _surface_hover alpha(@_fg, 0.14);
@define-color _surface_disabled alpha(@_fg, 0.04);
@define-color _subtext alpha(@_fg, 0.55);

window {{
    background-color: @_bg;
    padding: 16px;
}}

.app-label {{
    color: @_fg;
    font-size: 14px;
    font-weight: 500;
    margin-bottom: 4px;
}}

.volume-label {{
    color: @_subtext;
    font-size: 12px;
    margin-bottom: 8px;
}}

.stream-icon {{
    font-size: 15px;
    margin-right: 8px;
    color: @_fg;
}}

.stream-app-label {{
    color: @_fg;
    font-size: 14px;
    font-weight: 500;
}}

.stream-volume-label {{
    color: @_subtext;
    font-size: 12px;
    min-width: 36px;
}}

scale {{
    min-width: 200px;
    min-height: 6px;
    margin: 4px 0;
    margin-left: 0;
    padding-left: 0;
}}

scale slider {{
    background-color: @_accent;
    border-radius: 50%;
    min-width: 16px;
    min-height: 16px;
    border: none;
    box-shadow: none;
}}

scale trough {{
    background-color: @_surface;
    border-radius: 6px;
    min-height: 6px;
    border: none;
}}

scale highlight {{
    background-color: @_fg;
    border-radius: 6px;
}}

scale:disabled slider {{
    background-color: @_subtext;
}}

scale:disabled trough {{
    background-color: @_surface_disabled;
}}

scale:disabled highlight {{
    background-color: @_subtext;
}}

.section-title {{
    color: @_accent;
    font-size: 13px;
    font-weight: 600;
    margin-top: 12px;
    margin-bottom: 8px;
}}

.device-item {{
    color: @_subtext;
    padding: 8px 12px;
    margin: 2px;
    transition: background-color 0.2s ease;
}}

.device-item:hover {{
    background-color: @_surface_hover;
}}

.device-item.default {{
    background-color: @_surface;
    color: @_fg;
    
}}

.device-item.default:hover {{
    background-color: @_surface_hover;
}}

.device-icon {{
    font-size: 16px;
    margin-right: 8px;
}}

.device-label {{
    font-size: 13px;
    font-weight: 500;
}}

separator {{
    background-color: @_surface;
    min-height: 1px;
    margin: 12px 0;
}}

.container-box {{
    padding: 8px;
}}

.backdrop-capture {{
    background-color: rgba(0, 0, 0, 0.02);
    border-radius: 0;
    padding: 0;
}}

.settings-button {{
    background: @_surface;
    color: @_subtext;
    border: none;
    border-radius: 6px;
    padding: 6px 12px;
    font-size: 13px;
    margin-top: 4px;
}}

.settings-button:hover {{
    background: @_surface_hover;
    color: @_fg;
}}

"#
    )
}

/// Parse a `key = "value"` (TOML) or `@define-color key value;` (CSS) line from a file.
fn parse_color_from_file(path: &str, key: &str) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    for line in content.lines() {
        // CSS: @define-color foreground #ffcead;
        if let Some(rest) = line.strip_prefix("@define-color") {
            let rest = rest.trim().trim_end_matches(';');
            if let Some((k, v)) = rest.split_once(char::is_whitespace) {
                if k.trim() == key {
                    return Some(v.trim().to_string());
                }
            }
        }
        // TOML: accent = "#7d82d9"
        if let Some((k, v)) = line.split_once('=') {
            if k.trim() == key {
                return Some(v.trim().trim_matches('"').to_string());
            }
        }
    }
    None
}

pub fn setup_layer_shell(window: &ApplicationWindow) {
    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::OnDemand);
    window.set_anchor(gtk4_layer_shell::Edge::Top, true);
    window.set_anchor(gtk4_layer_shell::Edge::Right, true);
    window.set_margin(gtk4_layer_shell::Edge::Top, 10);
    window.set_margin(gtk4_layer_shell::Edge::Right, 10);
}

pub fn apply_css(window: &ApplicationWindow) {
    let provider = gtk::CssProvider::new();
    provider.load_from_data(&build_css());

    gtk::style_context_add_provider_for_display(
        &gtk::prelude::WidgetExt::display(window),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

pub fn build_ui(app: &Application, audio: Arc<Mutex<AudioManager>>) -> ApplicationWindow {
    let window = ApplicationWindow::builder()
        .application(app)
        .decorated(false)
        .resizable(false)
        .build();

    setup_layer_shell(&window);
    apply_css(&window);

    let main_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(0)
        .css_classes(vec!["container-box".to_string()])
        .build();

    let streams_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(4)
        .build();
    streams_box.set_widget_name("streams-box");

    let sinks_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(4)
        .build();
    sinks_box.set_widget_name("sinks-box");

    let sources_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(4)
        .build();
    sources_box.set_widget_name("sources-box");

    let streams_title = Label::builder()
        .label("Applications")
        .css_classes(vec!["section-title".to_string()])
        .halign(gtk::Align::Start)
        .build();
    main_box.append(&streams_title);
    main_box.append(&streams_box);

    let separator1 = Separator::builder()
        .orientation(Orientation::Horizontal)
        .build();
    main_box.append(&separator1);

    let sinks_title = Label::builder()
        .label("Playback Devices")
        .css_classes(vec!["section-title".to_string()])
        .halign(gtk::Align::Start)
        .build();
    main_box.append(&sinks_title);
    main_box.append(&sinks_box);

    let separator2 = Separator::builder()
        .orientation(Orientation::Horizontal)
        .build();
    main_box.append(&separator2);

    let sources_title = Label::builder()
        .label("Input Devices")
        .css_classes(vec!["section-title".to_string()])
        .halign(gtk::Align::Start)
        .build();
    main_box.append(&sources_title);
    main_box.append(&sources_box);

    let separator_settings = Separator::builder()
        .orientation(Orientation::Horizontal)
        .build();
    main_box.append(&separator_settings);

    let settings_button = Button::builder()
        .label("Settings")
        .css_classes(vec!["settings-button".to_string()])
        .build();
    settings_button.connect_clicked(|_| {
        let _ = std::process::Command::new("omarchy-launch-audio").spawn();
    });
    main_box.append(&settings_button);

    window.set_child(Some(&main_box));

    let audio_clone = audio.clone();
    let streams_box_clone = streams_box.clone();
    let sinks_box_clone = sinks_box.clone();
    let sources_box_clone = sources_box.clone();

    let audio_guard = audio.lock().unwrap();
    let streams_data = audio_guard.list_sink_inputs();
    let sinks_data = audio_guard.list_sinks();
    let sources_data = audio_guard.list_sources();
    drop(audio_guard);

    update_streams(&streams_box, &streams_data, audio_clone.clone());
    update_devices(&sinks_box, &sinks_data, audio_clone.clone(), true);
    update_devices(&sources_box, &sources_data, audio_clone.clone(), false);

    glib::timeout_add_seconds_local(2, move || {
        let audio_guard = audio_clone.lock().unwrap();
        let streams = audio_guard.list_sink_inputs();
        let sinks = audio_guard.list_sinks();
        let sources = audio_guard.list_sources();
        drop(audio_guard);

        update_streams(&streams_box_clone, &streams, audio_clone.clone());
        update_devices(&sinks_box_clone, &sinks, audio_clone.clone(), true);
        update_devices(&sources_box_clone, &sources, audio_clone.clone(), false);

        glib::ControlFlow::Continue
    });

    window.present();
    window
}

fn update_streams(container: &Box, streams: &[AudioStream], audio: Arc<Mutex<AudioManager>>) {
    // Group streams by app_name so each app gets a single slider
    let mut groups: std::collections::BTreeMap<String, Vec<&AudioStream>> =
        std::collections::BTreeMap::new();
    for stream in streams {
        // "Unknown" means APPLICATION_NAME was absent; keep each such stream
        // separate since they are unrelated apps that happen to lack a name.
        let key = if stream.app_name == "Unknown" {
            format!("Unknown\x1f{}", stream.index)
        } else {
            stream.app_name.clone()
        };
        groups.entry(key).or_default().push(stream);
    }

    let active_apps: std::collections::HashSet<&str> = groups.keys().map(|s| s.as_str()).collect();

    // Remove widgets for apps that are no longer active
    let mut to_remove = Vec::new();
    let mut child = container.first_child();
    while let Some(widget) = child {
        let widget_name = widget.widget_name();
        if let Some(app_name) = widget_name.strip_prefix("stream-app-") {
            if !active_apps.contains(app_name) {
                to_remove.push(widget.clone());
            }
        }
        child = widget.next_sibling();
    }
    for widget in to_remove {
        container.remove(&widget);
    }

    if groups.is_empty() {
        let mut has_placeholder = false;
        let mut child = container.first_child();
        while let Some(widget) = child {
            if widget.widget_name() == "placeholder" {
                has_placeholder = true;
                break;
            }
            child = widget.next_sibling();
        }

        if !has_placeholder {
            while let Some(child) = container.first_child() {
                container.remove(&child);
            }

            let label = Label::builder()
                .label("No applications playing audio")
                .css_classes(vec!["volume-label".to_string()])
                .halign(gtk::Align::Start)
                .build();
            label.set_widget_name("placeholder");
            container.append(&label);
        }
        return;
    }

    let placeholder = container.first_child();
    if let Some(ref widget) = placeholder {
        if widget.widget_name() == "placeholder" {
            container.remove(widget);
        }
    }

    for (app_name, app_streams) in &groups {
        let avg_volume = (app_streams.iter().map(|s| s.volume as u64).sum::<u64>()
            / app_streams.len() as u64) as u32;
        let indices: Vec<u32> = app_streams.iter().map(|s| s.index).collect();
        let widget_name = format!("stream-app-{}", app_name);

        let mut existing_widget = None;
        let mut child = container.first_child();
        while let Some(widget) = child {
            if widget.widget_name() == widget_name {
                existing_widget = Some(widget.clone());
                break;
            }
            child = widget.next_sibling();
        }

        if let Some(widget) = existing_widget {
            if let Some(stream_box) = widget.downcast_ref::<Box>() {
                // Update the captured indices so the slider controls any new/removed streams
                unsafe {
                    if let Some(ptr) = stream_box.data::<Rc<RefCell<Vec<u32>>>>("stream_indices") {
                        *ptr.as_ref().borrow_mut() = indices;
                    }
                }
                let children = stream_box.observe_children();
                // children: 0=header_box, 1=slider_row
                if let Some(slider_row_widget) = children.item(1) {
                    if let Some(slider_row) = slider_row_widget.downcast_ref::<Box>() {
                        let row_children = slider_row.observe_children();
                        // slider_row children: 0=scale, 1=volume_label
                        if let Some(scale_widget) = row_children.item(0) {
                            if let Some(scale) = scale_widget.downcast_ref::<Scale>() {
                                scale.adjustment().set_value(avg_volume as f64);
                            }
                        }
                        if let Some(vol_widget) = row_children.item(1) {
                            if let Some(volume_label) = vol_widget.downcast_ref::<Label>() {
                                volume_label.set_label(&format!("{}%", avg_volume));
                            }
                        }
                    }
                }
            }
        } else {
            let stream_box = Box::builder()
                .orientation(Orientation::Horizontal)
                .spacing(4)
                .build();
            stream_box.set_widget_name(&widget_name);

            // Row 1: icon
            let header_box = Box::builder()
                .orientation(Orientation::Horizontal)
                .spacing(0)
                .build();

            let display_name = &app_streams[0].app_name;

            let icon_label = Label::builder()
                .label(app_icon(display_name))
                .css_classes(vec!["stream-icon".to_string()])
                .build();

            header_box.append(&icon_label);

            // Row 2: slider + volume % inline
            let slider_row = Box::builder()
                .orientation(Orientation::Horizontal)
                .spacing(8)
                .valign(gtk::Align::Center)
                .build();

            let scale = Scale::builder()
                .orientation(Orientation::Horizontal)
                .adjustment(&gtk::Adjustment::new(
                    avg_volume as f64,
                    0.0,
                    100.0,
                    1.0,
                    10.0,
                    0.0,
                ))
                .hexpand(true)
                .draw_value(false)
                .build();

            let volume_label = Label::builder()
                .label(format!("{}%", avg_volume))
                .css_classes(vec!["stream-volume-label".to_string()])
                .halign(gtk::Align::End)
                .build();

            // Store indices in a shared cell so refresh cycles can update them
            // without recreating the widget (keeps slider stable during drags)
            let indices_cell = Rc::new(RefCell::new(indices));
            let indices_cell_clone = indices_cell.clone();
            unsafe {
                stream_box.set_data("stream_indices", indices_cell);
            }

            let audio_clone = audio.clone();
            let volume_label_clone = volume_label.clone();
            scale.connect_change_value(move |_scale, _scroll, value| {
                let value = value as u32;
                let audio = audio_clone.lock().unwrap();
                for &idx in indices_cell_clone.borrow().iter() {
                    audio.set_sink_input_volume(idx, value);
                }
                volume_label_clone.set_label(&format!("{}%", value));
                gtk::glib::Propagation::Proceed
            });

            slider_row.append(&scale);
            slider_row.append(&volume_label);

            // children order: 0=header_box, 1=slider_row
            stream_box.append(&header_box);
            stream_box.append(&slider_row);
            container.append(&stream_box);
        }
    }
}

fn update_devices(
    container: &Box,
    devices: &[AudioDevice],
    audio: Arc<Mutex<AudioManager>>,
    is_sink: bool,
) {
    let existing_indices: std::collections::HashSet<u32> =
        devices.iter().map(|d| d.index).collect();

    // Remove widgets for devices that disappeared and any stale placeholder
    let mut to_remove = Vec::new();
    let mut child = container.first_child();
    while let Some(widget) = child {
        let wname = widget.widget_name();
        if wname == "placeholder" {
            if !devices.is_empty() {
                to_remove.push(widget.clone());
            }
        } else if let Some(idx_str) = wname.strip_prefix("device-") {
            if let Ok(idx) = idx_str.parse::<u32>() {
                if !existing_indices.contains(&idx) {
                    to_remove.push(widget.clone());
                }
            }
        }
        child = widget.next_sibling();
    }
    for widget in to_remove {
        container.remove(&widget);
    }

    if devices.is_empty() {
        // Only add placeholder if not already there
        let has_placeholder = container
            .first_child()
            .map_or(false, |w| w.widget_name() == "placeholder");
        if !has_placeholder {
            let label = Label::builder()
                .label(if is_sink {
                    "No playback devices found"
                } else {
                    "No input devices found"
                })
                .css_classes(vec!["volume-label".to_string()])
                .halign(gtk::Align::Start)
                .build();
            label.set_widget_name("placeholder");
            container.append(&label);
        }
        return;
    }

    for device in devices.iter() {
        let widget_name = format!("device-{}", device.index);

        // Find existing widget for this device
        let mut existing = None;
        let mut child = container.first_child();
        while let Some(widget) = child {
            if widget.widget_name() == widget_name {
                existing = Some(widget.clone());
                break;
            }
            child = widget.next_sibling();
        }

        if let Some(widget) = existing {
            // Only update the default CSS class if it changed
            if let Some(item_box) = widget.downcast_ref::<Box>() {
                if device.is_default {
                    item_box.add_css_class("default");
                } else {
                    item_box.remove_css_class("default");
                }
            }
        } else {
            let mut css_classes = vec!["device-item".to_string()];
            if device.is_default {
                css_classes.push("default".to_string());
            }

            let item_box = Box::builder()
                .orientation(Orientation::Horizontal)
                .spacing(0)
                .css_classes(css_classes)
                .build();
            item_box.set_widget_name(&widget_name);

            let icon = Label::builder()
                .label(if is_sink { "\u{f057f}" } else { "\u{f130}" })
                .css_classes(vec!["device-icon".to_string()])
                .build();

            let text = Label::builder()
                .label(&device.description)
                .css_classes(vec!["device-label".to_string()])
                .halign(gtk::Align::Start)
                .build();

            item_box.append(&icon);
            item_box.append(&text);

            let name = device.name.clone();
            let audio_clone = audio.clone();
            let container_clone = container.clone();
            let item_box_clone = item_box.clone();
            let gesture = gtk::GestureClick::new();
            gesture.connect_pressed(move |_, _, _, _| {
                // Optimistic update: reflect the change immediately without waiting for the timer
                let mut child = container_clone.first_child();
                while let Some(widget) = child {
                    if let Some(b) = widget.downcast_ref::<Box>() {
                        b.remove_css_class("default");
                    }
                    child = widget.next_sibling();
                }
                item_box_clone.add_css_class("default");

                let audio = audio_clone.lock().unwrap();
                if is_sink {
                    audio.set_default_sink(&name);
                } else {
                    audio.set_default_source(&name);
                }
            });
            item_box.add_controller(gesture);

            container.append(&item_box);
        }
    }
}
