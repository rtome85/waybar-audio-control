mod audio;
mod ui;

use audio::AudioManager;
use gtk::prelude::*;
use gtk::Application;
use gtk4 as gtk;
use gtk4::gio;
use gtk4_layer_shell::{Layer, LayerShell};
use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

const PID_FILE: &str = "/tmp/audio-control.pid";

fn pid_file() -> PathBuf {
    PathBuf::from(PID_FILE)
}

/// Send SIGUSR1 to an existing instance. Returns true if successful.
fn try_signal_existing() -> bool {
    if let Ok(content) = fs::read_to_string(pid_file()) {
        if let Ok(pid) = content.trim().parse::<i32>() {
            unsafe {
                return libc::kill(pid, libc::SIGUSR1) == 0;
            }
        }
    }
    false
}

fn main() {
    // If an existing instance is running, signal it and exit immediately
    if try_signal_existing() {
        return;
    }

    // Write our PID so future invocations can find us
    let _ = fs::write(pid_file(), std::process::id().to_string());

    let app = Application::builder()
        .application_id("com.waybar.audio-control")
        // NON_UNIQUE: skip GTK's D-Bus single-instance — we handle IPC ourselves
        .flags(gio::ApplicationFlags::NON_UNIQUE)
        .build();

    // Keep the process alive even when all windows are hidden
    let _hold = app.hold();

    let windows: Rc<RefCell<Option<(gtk::ApplicationWindow, gtk::ApplicationWindow)>>> =
        Rc::new(RefCell::new(None));

    app.connect_activate({
        let windows = windows.clone();
        move |app| {
            // First activation: build everything
            let audio = match AudioManager::connect() {
                Ok(manager) => Arc::new(Mutex::new(manager)),
                Err(e) => {
                    eprintln!("Failed to connect to PulseAudio: {:?}", e);
                    return;
                }
            };

            let popup = ui::build_ui(app, audio.clone());
            let backdrop = gtk::ApplicationWindow::builder()
                .application(app)
                .decorated(false)
                .build();

            // Backdrop fullscreen em Layer::Top que captura cliques fora do popup.
            // Usa rgba(0,0,0,0.02) para forçar o GTK a commitar um buffer Wayland
            // com alpha não-zero — necessário para o Hyprland encaminhar eventos de input.
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
                    popup.hide();
                    backdrop.hide();
                }
            });
            backdrop.add_controller(gesture);

            popup.connect_close_request(|w| {
                w.hide();
                glib::Propagation::Stop
            });
            backdrop.connect_close_request(|w| {
                w.hide();
                glib::Propagation::Stop
            });

            backdrop.present();
            *windows.borrow_mut() = Some((popup, backdrop));

            // SIGUSR1 toggles visibility — used by subsequent waybar clicks
            glib::unix_signal_add_local(libc::SIGUSR1, {
                let windows = windows.clone();
                move || {
                    let state = windows.borrow();
                    if let Some((ref popup, ref backdrop)) = *state {
                        if popup.is_visible() {
                            popup.hide();
                            backdrop.hide();
                        } else {
                            backdrop.present();
                            popup.present();
                        }
                    }
                    glib::ControlFlow::Continue
                }
            });
        }
    });

    app.connect_shutdown(|_| {
        let _ = fs::remove_file(pid_file());
    });

    app.run();
    let _ = fs::remove_file(pid_file());
}
