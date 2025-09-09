use rusb::{Context, Device, Hotplug, HotplugBuilder, UsbContext};

struct HotPlugHandler;

impl Hotplug<Context> for HotPlugHandler {
    fn device_arrived(&mut self, device: Device<Context>) {
        println!("Dispositivo conectado: {:?}", device.device_descriptor().unwrap());
    }

    fn device_left(&mut self, device: Device<Context>) {
        println!("Dispositivo removido: {:?}", device.device_descriptor().unwrap());
    }
}

impl Drop for HotPlugHandler {
    fn drop(&mut self) {
        println!("HotPlugHandler finalizado");
    }
}

fn main() {
    if !rusb::has_hotplug() {
        eprintln!("libusb hotplug não suportado nesta versão.");
        return;
    }

    let context = Context::new().expect("Erro ao criar contexto USB");

    let _registration = HotplugBuilder::new()
        .enumerate(true) // dispara também para dispositivos já conectados
        .register(&context, Box::new(HotPlugHandler {}))
        .expect("Falha ao registrar callback hotplug");

    println!("Monitorando eventos USB... Ctrl+C para sair.");

    loop {
        // processa eventos de hotplug — bloqueia de forma leve
        context.handle_events(None).unwrap();
    }
}
