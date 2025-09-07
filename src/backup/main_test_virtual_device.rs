mod config;
mod hotplug;
mod reader;
mod virtual_device; // 👈 novo módulo

use crate::{
    config::Config,
    hotplug::{HotPlugHandler, hotplug::CustomHotplugEvent},
    reader::USBReader,
    virtual_device::{VirtualPen, VirtualButtons}, // 👈 importa structs
};

use std::path::Path;

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

    // cria os devices virtuais
    let mut vpen = VirtualPen::new(
        &cfg.xinput_name,
        cfg.pen.max_x,
        cfg.pen.max_y,
        cfg.pen.max_pressure,
        cfg.pen.resolution_x,
        cfg.pen.resolution_y,
        &cfg.actions.pen_codes, // precisa converter para Vec<i32>
    ).expect("Erro criando VirtualPen");

    let mut vbtn = VirtualButtons::new(
        &(cfg.xinput_name.clone() + "_buttons"),
        &cfg.actions.btn_codes, // idem, precisa ser Vec<i32>
    ).expect("Erro criando VirtualButtons");

    HotPlugHandler::init(move |device, event| match event {
        CustomHotplugEvent::DeviceArrived => {
            let vendor_id: u16 = 0x08f2;
            let product_id: u16 = 0x6811;
            let endpoint: u8 = 0x85;

            let usb_reader = USBReader::new().unwrap();

            println!(
                "Callback: dispositivo conectado {:?}",
                device.device_descriptor().unwrap()
            );

            let desc = device.device_descriptor().unwrap();

            if desc.vendor_id() == vendor_id && desc.product_id() == product_id {
                println!("Tablet detectado, iniciando leitura...");

                // move clones dos devices virtuais para dentro do closure
                let mut vpen_clone = vpen.clone();
                let mut vbtn_clone = vbtn.clone();

                usb_reader.start(device, endpoint, move |buf| {
                    println!("Recebi pacote: {:?}", buf);

                    // TODO: aqui no futuro faz parsing de buf e escreve no VirtualPen/VirtualButtons
                    // Exemplo fake só pra ver os eventos chegando:
                    if !buf.is_empty() {
                        let _ = vpen_clone.write_abs(0, buf[0] as i32);
                        let _ = vbtn_clone.write_key(30, 1); // KEY_A down
                        let _ = vbtn_clone.write_key(30, 0); // KEY_A up
                        let _ = vpen_clone.sync();
                        let _ = vbtn_clone.sync();
                    }
                }).unwrap();
            }
        }
        CustomHotplugEvent::DeviceLeft => {
            println!(
                "Callback: dispositivo desconectado {:?}",
                device.device_descriptor().unwrap()
            );
        }
    });

    println!("Main continua rodando o loop principal...");

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
