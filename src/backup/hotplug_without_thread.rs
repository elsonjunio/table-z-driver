//use rusb::{Context, Device, Hotplug, HotplugBuilder, HotplugEvent, UsbContext};
use rusb::{Context, Device, Hotplug, HotplugBuilder, UsbContext};
use std::sync::{Arc, Mutex};

/// Enum próprio para eventos de hotplug
#[derive(Debug, Clone, Copy)]
pub enum CustomHotplugEvent {
    DeviceArrived,
    DeviceLeft,
}

pub struct HotPlugHandler {
    callback: Arc<Mutex<dyn FnMut(Device<Context>, CustomHotplugEvent) + Send>>,
}

impl Hotplug<Context> for HotPlugHandler {
    fn device_arrived(&mut self, device: Device<Context>) {
        (self.callback.lock().unwrap())(device, CustomHotplugEvent::DeviceArrived);
    }

    fn device_left(&mut self, device: Device<Context>) {
        (self.callback.lock().unwrap())(device, CustomHotplugEvent::DeviceLeft);
    }
}

impl HotPlugHandler {
    pub fn init<F>(callback: F)
    where
        F: FnMut(Device<Context>, CustomHotplugEvent) + Send + 'static,
    {
        if !rusb::has_hotplug() {
            eprintln!("libusb hotplug não suportado nesta versão.");
            return;
        }

        let context = Context::new().expect("Erro ao criar contexto USB");

        let cb = Arc::new(Mutex::new(callback));

        let _registration = HotplugBuilder::new()
            .enumerate(true)
            .register(&context, Box::new(HotPlugHandler { callback: cb }))
            .expect("Falha ao registrar callback hotplug");

        println!("Monitorando eventos USB... Ctrl+C para sair.");

        loop {
            if let Err(e) = context.handle_events(None) {
                eprintln!("Erro no handle_events: {:?}", e);
                break;
            }
        }
    }
}
