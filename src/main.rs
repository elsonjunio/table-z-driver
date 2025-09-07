mod config;
mod hotplug;
mod reader;
mod virtual_device;

use crate::{
    config::Config,
    hotplug::HotPlugHandler,
    hotplug::hotplug::CustomHotplugEvent,
    reader::USBReader,
    virtual_device::{VBtn, VPen},
};
use std::path::Path;

use evdev::Key;

use std::sync::{Arc, Mutex};

fn main() {
    let path = Path::new("/etc/table_z_utils.yaml");

    let cfg = match Config::from_file(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Erro ao carregar configuração: {:?}", e);
            return;
        }
    };

    println!("Configuração carregada:\n{:#?}", cfg);

    HotPlugHandler::init(|device, event| match event {
        CustomHotplugEvent::DeviceArrived => {
            let vendor_id: u16 = 0x08f2;
            let product_id: u16 = 0x6811;
            let endpoint: u8 = 0x85;

            let usb_reader = USBReader::new().unwrap();

            let pen_keys = [Key::BTN_TOOL_PEN, Key::BTN_STYLUS, Key::BTN_TOUCH];
            let mut vpen = VPen::new(4096, 4096, 2047, 20, 30, &pen_keys, "10moons-pen").unwrap();

            println!(
                "Callback: dispositivo conectado {:?}",
                device.device_descriptor().unwrap()
            );

            let desc = device.device_descriptor().unwrap();

            if desc.vendor_id() == vendor_id && desc.product_id() == product_id {
                println!("Dispositivo encontrado!");

                // clona o VPen para usar no callback
                let vpen_clone = vpen.clone();

                usb_reader
                    .start(device, endpoint, move |buf| {
                        println!("Recebi pacote: {:?}", buf);

                        if buf.len() >= 8 && (buf[1] == 192 || buf[1] == 193) {
                            let pen_x = (buf[5] as i32 * 255 + buf[4] as i32);
                            let pen_y = (buf[3] as i32 * 255 + buf[2] as i32);
                            let pen_pressure = (buf[7] as i32 * 255 + buf[6] as i32);
                            let touching = buf[1] != 192;

                            if let Err(e) = vpen_clone.emit(pen_x, pen_y, pen_pressure, touching) {
                                eprintln!("Erro emitindo evento: {e}");
                            }
                        }
                    })
                    .unwrap();
            }
            //USBReader::new(device, endpoint)
        }
        CustomHotplugEvent::DeviceLeft => {
            println!(
                "Callback: dispositivo desconectado {:?}",
                device.device_descriptor().unwrap()
            );
        }
    });

    println!("Main continua rodando o loop principal...");

    //    let btn_keys = [Key::KEY_LEFTCTRL, Key::KEY_Z, Key::KEY_C];
    //    let mut vbtn = VBtn::new(&btn_keys, "10moons-pen-buttons").unwrap();

    // Exemplo: move a caneta
    //vpen.emit(1000, 2000, 500, true).unwrap();
    //vpen.emit(750, 1500, 500, true).unwrap();
    //vpen.emit(500, 1000, 500, true).unwrap();
    //vpen.emit(250, 500, 500, true).unwrap();

    // Simula alguns movimentos e toques
    //vpen.emit(1000, 2000, 500, true).unwrap();
    //vpen.emit(2000, 3000, 1000, true).unwrap();
    //vpen.emit(3000, 1000, 1500, false).unwrap();

    // Exemplo: pressiona um botão
    //vbtn.emit(Key::KEY_C, 1).unwrap(); // 1 = pressionado
    //vbtn.emit(Key::KEY_C, 0).unwrap(); // 0 = liberado

    loop {
        // outras lógicas da aplicação (socket, config, etc.)
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

// buf: &mut [u8]
