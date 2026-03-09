use libpulse_binding::{
    context::{Context, FlagSet},
    mainloop::standard::Mainloop,
    proplist::Proplist,
    volume::{ChannelVolumes, VolumeLinear},
};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct AudioStream {
    pub index: u32,
    pub name: String,
    pub volume: u32,
    pub app_name: String,
}

#[derive(Clone, Debug)]
pub struct AudioDevice {
    pub index: u32,
    pub name: String,
    pub description: String,
    pub is_default: bool,
}

pub struct AudioManager {
    mainloop: Rc<RefCell<Mainloop>>,
    context: Rc<RefCell<Context>>,
}

impl AudioManager {
    pub fn connect() -> Result<Self, Box<dyn std::error::Error>> {
        let mut proplist = Proplist::new().ok_or("Failed to create proplist")?;
        let _ = proplist.set(
            libpulse_binding::proplist::properties::APPLICATION_NAME,
            b"audio-control",
        );

        let mainloop = Rc::new(RefCell::new(
            Mainloop::new().ok_or("Failed to create mainloop")?,
        ));

        let mainloop_ref = mainloop.borrow();
        let context = Rc::new(RefCell::new(
            Context::new_with_proplist(&*mainloop_ref, "audio-control", &proplist)
                .ok_or("Failed to create context")?,
        ));
        drop(mainloop_ref);

        context.borrow_mut().connect(None, FlagSet::NOFLAGS, None)?;

        loop {
            match mainloop.borrow_mut().iterate(false) {
                libpulse_binding::mainloop::standard::IterateResult::Success(_) => {}
                libpulse_binding::mainloop::standard::IterateResult::Quit(_) => {
                    break;
                }
                libpulse_binding::mainloop::standard::IterateResult::Err(e) => {
                    return Err(format!("Mainloop error: {:?}", e).into());
                }
            }

            if context.borrow().get_state() == libpulse_binding::context::State::Ready {
                break;
            }
        }

        Ok(Self { mainloop, context })
    }

    pub fn list_sink_inputs(&self) -> Vec<AudioStream> {
        let streams = Rc::new(RefCell::new(Vec::new()));
        let streams_clone = streams.clone();

        let introspector = self.context.borrow().introspect();

        introspector.get_sink_input_info_list(move |result| {
            if let libpulse_binding::callbacks::ListResult::Item(info) = result {
                let mut result_vec = streams_clone.borrow_mut();

                let volume_percent = if info.volume.len() > 0 {
                    let avg = info.volume.avg();
                    ((VolumeLinear::from(avg).0 as f32) * 100.0) as u32
                } else {
                    0
                };

                let app_name = info
                    .proplist
                    .get(libpulse_binding::proplist::properties::APPLICATION_NAME)
                    .and_then(|bytes| {
                        let bytes = bytes.strip_suffix(b"\0").unwrap_or(bytes);
                        std::str::from_utf8(bytes).ok()
                    })
                    .unwrap_or("Unknown")
                    .to_string();

                result_vec.push(AudioStream {
                    index: info.index,
                    name: info
                        .name
                        .as_ref()
                        .map(|s| s.to_string())
                        .unwrap_or_default(),
                    volume: volume_percent,
                    app_name,
                });
            }
        });

        self.iterate_until_complete();

        let result = streams.borrow().clone();
        result
    }

    pub fn list_sinks(&self) -> Vec<AudioDevice> {
        let devices = Rc::new(RefCell::new(Vec::new()));
        let devices_clone = devices.clone();

        let introspector = self.context.borrow().introspect();

        introspector.get_sink_info_list(move |result| {
            if let libpulse_binding::callbacks::ListResult::Item(info) = result {
                let mut result_vec = devices_clone.borrow_mut();
                result_vec.push(AudioDevice {
                    index: info.index,
                    name: info
                        .name
                        .as_ref()
                        .map(|s| s.to_string())
                        .unwrap_or_default(),
                    description: info
                        .description
                        .as_ref()
                        .map(|s| s.to_string())
                        .unwrap_or_default(),
                    is_default: false,
                });
            }
        });

        self.iterate_until_complete();

        let result = devices.borrow().clone();
        result
    }

    pub fn list_sources(&self) -> Vec<AudioDevice> {
        let devices = Rc::new(RefCell::new(Vec::new()));
        let devices_clone = devices.clone();

        let introspector = self.context.borrow().introspect();

        introspector.get_source_info_list(move |result| {
            if let libpulse_binding::callbacks::ListResult::Item(info) = result {
                let mut result_vec = devices_clone.borrow_mut();
                result_vec.push(AudioDevice {
                    index: info.index,
                    name: info
                        .name
                        .as_ref()
                        .map(|s| s.to_string())
                        .unwrap_or_default(),
                    description: info
                        .description
                        .as_ref()
                        .map(|s| s.to_string())
                        .unwrap_or_default(),
                    is_default: false,
                });
            }
        });

        self.iterate_until_complete();

        let result = devices.borrow().clone();
        result
    }

    pub fn set_sink_input_volume(&self, index: u32, volume_percent: u32) {
        let volume_linear = (volume_percent as f64) / 100.0;
        let volume_pa: libpulse_binding::volume::Volume = VolumeLinear(volume_linear).into();
        let mut cvol = ChannelVolumes::default();
        cvol.set(2, volume_pa);

        let mut introspector = self.context.borrow().introspect();
        introspector.set_sink_input_volume(index, &cvol, None);

        self.iterate_until_complete();
    }

    pub fn set_default_sink(&self, name: &str) {
        self.context
            .borrow_mut()
            .set_default_sink(name, |_: bool| {});

        self.iterate_until_complete();
    }

    pub fn set_default_source(&self, name: &str) {
        self.context
            .borrow_mut()
            .set_default_source(name, |_: bool| {});

        self.iterate_until_complete();
    }

    fn iterate_until_complete(&self) {
        for _ in 0..100 {
            match self.mainloop.borrow_mut().iterate(false) {
                libpulse_binding::mainloop::standard::IterateResult::Success(_) => {}
                _ => break,
            }
        }
    }
}
