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

use std::sync::{
    Arc, Mutex, OnceLock,
    atomic::{AtomicBool, Ordering},
};

use std::collections::HashMap;

mod com;

use com::socket::SocketServer;

static STOP_FLAG: OnceLock<Mutex<Option<Arc<AtomicBool>>>> = OnceLock::new();

fn init_globals() {
    // inicializa o OnceLock na primeira chamada
    STOP_FLAG.get_or_init(|| Mutex::new(None));
}

fn key_from_str(name: &str) -> Option<Key> {
    let map: HashMap<&'static str, Key> = [
        ("BTN_TOOL_PEN", Key::BTN_TOOL_PEN),
        ("BTN_STYLUS", Key::BTN_STYLUS),
        ("BTN_TOUCH", Key::BTN_TOUCH),
        // se precisar de mais, só adicionar
    ]
    .into_iter()
    .collect();

    map.get(name).copied()
}

fn main() {
    init_globals();

    let socket_server = SocketServer::new("/tmp/tablet.sock");
    let tx_socket = socket_server.sender();

    let path = Path::new("/etc/table_z_utils.yaml");

    let cfg = match Config::from_file(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Erro ao carregar configuração: {:?}", e);
            return;
        }
    };

    println!("Configuração carregada:\n{:#?}", cfg);

    HotPlugHandler::init({
        let cfg = cfg.clone(); // precisa do derive Clone em Config

        move |device, event| match event {
            CustomHotplugEvent::DeviceArrived => {
                // pega dados da configuração
                let vendor_id: u16 = cfg.vendor_id;
                let product_id: u16 = cfg.product_id;
                let endpoint: u8 = cfg.interface;

                println!(
                    "Callback: dispositivo conectado {:?}",
                    device.device_descriptor().unwrap()
                );

                let desc = device.device_descriptor().unwrap();

                if desc.vendor_id() == vendor_id && desc.product_id() == product_id {
                    println!("Dispositivo encontrado!");

                    // Instancia Objetos
                    let usb_reader = USBReader::new().unwrap();
                    let stop_flag = Arc::new(AtomicBool::new(true));

                    let pen_keys: Vec<Key> = vec![
                        key_from_str(&cfg.actions.pen).unwrap(),
                        key_from_str(&cfg.actions.stylus).unwrap(),
                        key_from_str(&cfg.actions.pen_touch).unwrap(),
                    ];

                    let mut vpen = VPen::new(
                        cfg.pen.max_x as i32,
                        cfg.pen.max_y as i32,
                        cfg.pen.max_pressure as i32,
                        cfg.pen.resolution_x as i32,
                        cfg.pen.resolution_y as i32,
                        &pen_keys,
                        &cfg.xinput_name,
                    )
                    .unwrap();

                    {
                        let mut guard = STOP_FLAG.get_or_init(|| Mutex::new(None)).lock().unwrap();
                        *guard = Some(stop_flag.clone());
                    }

                    // Clona o VPen para usar no callback
                    let vpen_clone = vpen.clone();
                    let tx_socket = tx_socket.clone();
                    //let stop_clone = stop_flag.clone();

                    usb_reader
                        .start(device, endpoint, stop_flag.clone(), move |buf| {
                            println!("Recebi pacote: {:?}", buf);

                            if buf.len() >= 8 && (buf[1] == 192 || buf[1] == 193) {
                                let pen_x = (buf[5] as i32 * 255 + buf[4] as i32);
                                let pen_y = (buf[3] as i32 * 255 + buf[2] as i32);
                                let pen_pressure = (buf[7] as i32 * 255 + buf[6] as i32);
                                let touching = buf[1] != 192;

                                if let Err(e) =
                                    vpen_clone.emit(pen_x, pen_y, pen_pressure, touching)
                                {
                                    eprintln!("Erro emitindo evento: {e}");
                                }

                                let _ = tx_socket.send(buf.clone());
                            }
                        })
                        .unwrap();
                }
            }
            CustomHotplugEvent::DeviceLeft => {
                println!(
                    "Callback: dispositivo desconectado {:?}",
                    device.device_descriptor().unwrap()
                );

                if let Some(flag) = STOP_FLAG
                    .get_or_init(|| Mutex::new(None))
                    .lock()
                    .unwrap()
                    .take()
                {
                    flag.store(false, Ordering::SeqCst);
                    println!("Dispositivo desconectado: reader parado.");
                }
            }
        }
    });

    println!("Main continua rodando o loop principal...");

    loop {
        if let Some(cmd) = socket_server.try_recv_command() {
            println!("Comando recebido via socket: {}", cmd);
        }
        // outras lógicas da aplicação (socket, config, etc.)
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
