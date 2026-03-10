use crate::audio::{AudioDevice, AudioManager, AudioStream};
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box, Label, Orientation, Scale, Separator};
use gtk4 as gtk;
use gtk4_layer_shell::{Layer, LayerShell};
use std::sync::{Arc, Mutex};

const CSS: &str = r#"
window {
    background-color: #1e1e2e;
    border-radius: 12px;
    padding: 16px;
}

.app-label {
    color: #cdd6f4;
    font-size: 14px;
    font-weight: 500;
    margin-bottom: 4px;
}

.volume-label {
    color: #a6adc8;
    font-size: 12px;
    margin-bottom: 8px;
}

scale {
    min-width: 280px;
    min-height: 6px;
    margin: 4px 0;
}

scale slider {
    background-color: #f5c2e7;
    border-radius: 50%;
    min-width: 16px;
    min-height: 16px;
    border: none;
    box-shadow: none;
}

scale trough {
    background-color: #313244;
    border-radius: 6px;
    min-height: 6px;
    border: none;
}

scale highlight {
    background-color: #f5c2e7;
    border-radius: 6px;
}

scale:disabled slider {
    background-color: #585b70;
}

scale:disabled trough {
    background-color: #45475a;
}

scale:disabled highlight {
    background-color: #585b70;
}

.section-title {
    color: #f5c2e7;
    font-size: 13px;
    font-weight: 600;
    margin-top: 12px;
    margin-bottom: 8px;
}

.device-item {
    background-color: #313244;
    color: #cdd6f4;
    border-radius: 8px;
    padding: 8px 12px;
    margin: 2px;
    transition: background-color 0.2s ease;
}

.device-item:hover {
    background-color: #45475a;
}

.device-item.default {
    background-color: #f5c2e7;
    color: #1e1e2e;
}

.device-item.default:hover {
    background-color: #ebaac0;
}

.device-icon {
    font-size: 16px;
    margin-right: 8px;
}

.device-label {
    font-size: 13px;
    font-weight: 500;
}

separator {
    background-color: #313244;
    min-height: 1px;
    margin: 12px 0;
}

.container-box {
    padding: 8px;
}

window.backdrop-capture {
    background-color: rgba(0, 0, 0, 0.02);
    border-radius: 0;
    padding: 0;
}
"#;

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
    provider.load_from_data(CSS);

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
    let mut existing_indices = std::collections::HashSet::new();

    for stream in streams {
        existing_indices.insert(stream.index);
    }

    let mut to_remove = Vec::new();
    let mut child = container.first_child();
    while let Some(widget) = child {
        let widget_name = widget.widget_name();
        if widget_name.starts_with("stream-") {
            if let Ok(index_str) = widget_name
                .strip_prefix("stream-")
                .unwrap_or("")
                .parse::<u32>()
            {
                if !existing_indices.contains(&index_str) {
                    to_remove.push(widget.clone());
                }
            }
        }
        child = widget.next_sibling();
    }

    for widget in to_remove {
        container.remove(&widget);
    }

    if streams.is_empty() {
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

    for stream in streams {
        let widget_name = format!("stream-{}", stream.index);

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
                let children = stream_box.observe_children();
                if let Some(volume_label_widget) = children.item(1) {
                    if let Some(volume_label) = volume_label_widget.downcast_ref::<Label>() {
                        volume_label.set_label(&format!("{}%", stream.volume));
                    }
                }
                if let Some(scale_widget) = children.item(2) {
                    if let Some(scale) = scale_widget.downcast_ref::<Scale>() {
                        let adj = scale.adjustment();
                        adj.set_value(stream.volume as f64);
                    }
                }
            }
        } else {
            let stream_box = Box::builder()
                .orientation(Orientation::Vertical)
                .spacing(0)
                .margin_bottom(8)
                .build();
            stream_box.set_widget_name(&widget_name);

            let app_label = Label::builder()
                .label(&stream.app_name)
                .css_classes(vec!["app-label".to_string()])
                .halign(gtk::Align::Start)
                .build();

            let volume_label = Label::builder()
                .label(format!("{}%", stream.volume))
                .css_classes(vec!["volume-label".to_string()])
                .halign(gtk::Align::Start)
                .build();

            let scale = Scale::builder()
                .orientation(Orientation::Horizontal)
                .adjustment(&gtk::Adjustment::new(
                    stream.volume as f64,
                    0.0,
                    100.0,
                    1.0,
                    10.0,
                    0.0,
                ))
                .hexpand(true)
                .draw_value(false)
                .build();

            let index = stream.index;
            let audio_clone = audio.clone();
            let volume_label_clone = volume_label.clone();
            scale.connect_value_changed(move |scale| {
                let value = scale.value() as u32;
                let audio = audio_clone.lock().unwrap();
                audio.set_sink_input_volume(index, value);
                volume_label_clone.set_label(&format!("{}%", value));
            });

            stream_box.append(&app_label);
            stream_box.append(&volume_label);
            stream_box.append(&scale);
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
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }

    if devices.is_empty() {
        let label = Label::builder()
            .label(if is_sink {
                "No playback devices found"
            } else {
                "No input devices found"
            })
            .css_classes(vec!["volume-label".to_string()])
            .halign(gtk::Align::Start)
            .build();
        container.append(&label);
        return;
    }

    let devices_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(4)
        .build();

    for device in devices.iter() {
        let mut css_classes = vec!["device-item".to_string()];
        if device.is_default {
            css_classes.push("default".to_string());
        }

        let item_box = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(0)
            .css_classes(css_classes)
            .build();

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
        let gesture = gtk::GestureClick::new();
        gesture.connect_pressed(move |_, _, _, _| {
            let audio = audio_clone.lock().unwrap();
            if is_sink {
                audio.set_default_sink(&name);
            } else {
                audio.set_default_source(&name);
            }
        });
        item_box.add_controller(gesture);

        devices_box.append(&item_box);
    }

    container.append(&devices_box);
}
