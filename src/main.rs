mod audio;
mod ui;

use audio::AudioManager;
use gtk::prelude::*;
use gtk::Application;
use gtk4 as gtk;
use gtk4_layer_shell::{Layer, LayerShell};
use std::sync::{Arc, Mutex};

fn main() {
    let app = Application::builder()
        .application_id("com.waybar.audio-control")
        .build();

    app.connect_activate(|app| {
        let audio = match AudioManager::connect() {
            Ok(manager) => Arc::new(Mutex::new(manager)),
            Err(e) => {
                eprintln!("Failed to connect to PulseAudio: {:?}", e);
                return;
            }
        };

        // Popup em Layer::Overlay (acima de tudo, incluindo o backdrop)
        let popup = ui::build_ui(app, audio.clone());

        // Backdrop fullscreen em Layer::Top que captura cliques fora do popup.
        // Usa rgba(0,0,0,0.02) para forçar o GTK a commitar um buffer Wayland
        // com alpha não-zero — necessário para o Hyprland encaminhar eventos de input.
        let backdrop = gtk::ApplicationWindow::builder()
            .application(app)
            .decorated(false)
            .build();

        backdrop.init_layer_shell();
        backdrop.set_layer(Layer::Top);
        backdrop.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::None);
        backdrop.set_anchor(gtk4_layer_shell::Edge::Top, true);
        backdrop.set_anchor(gtk4_layer_shell::Edge::Bottom, true);
        backdrop.set_anchor(gtk4_layer_shell::Edge::Left, true);
        backdrop.set_anchor(gtk4_layer_shell::Edge::Right, true);
        backdrop.add_css_class("backdrop-capture");

        // Filho vazio garante que o GTK faz layout e commita o buffer
        let empty = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        backdrop.set_child(Some(&empty));

        let gesture = gtk::GestureClick::new();
        gesture.connect_pressed({
            let popup = popup.clone();
            let backdrop = backdrop.clone();
            move |_, _, _, _| {
                popup.close();
                backdrop.close();
            }
        });
        backdrop.add_controller(gesture);
        backdrop.present();
    });

    app.run();
}
