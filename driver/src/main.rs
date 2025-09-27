mod com;
mod config;
mod hotplug;
mod reader;
mod translator;
mod virtual_device;

use std::str::FromStr;
use evdev::{EvdevEnum, Key};

use crate::{
    com::socket::SocketServer,
    config::Config,
    hotplug::{HotPlugHandler, hotplug::CustomHotplugEvent},
    reader::USBReader,
    translator::{
        tablet_m100_translator::TabletM100Translator, translator::EmitCommand,
        translator::Translator,
    },
    virtual_device::{VBtn, VPen},
};
use std::path::Path;
//use bincode;

use serde::{Serialize, Deserialize};
use serde_json;

use std::sync::{
    Arc, Mutex, OnceLock,
    atomic::{AtomicBool, Ordering},
};

use std::collections::HashMap;

static STOP_FLAG: OnceLock<Mutex<Option<Arc<AtomicBool>>>> = OnceLock::new();

fn init_globals() {
    // inicializa o OnceLock na primeira chamada
    STOP_FLAG.get_or_init(|| Mutex::new(None));
}

fn all_keys() -> Vec<Key> {
    // keycodes válidos vão de 0 até 0x2FF
    (0..=0x2FFu16)
        .map(Key::new)
        .collect()
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
                        Key::from_str(&cfg.actions.pen).unwrap(),
                        Key::from_str(&cfg.actions.stylus).unwrap(),
                        Key::from_str(&cfg.actions.pen_touch).unwrap(),
                    ];

                    let vpen = VPen::new(
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


                    //--
                    let keys = all_keys();
                    let vbtn = VBtn::new(&keys, &cfg.xinput_name).unwrap();

                    // Clona o VPen para usar no callback
                    let vbtn_clone = vbtn.clone();
                    let vpen_clone = vpen.clone();
                    let tx_socket = tx_socket.clone();
                    //let stop_clone = stop_flag.clone();

                    
                    let translator = TabletM100Translator::new(Arc::new(Mutex::new(cfg.clone())));
                    usb_reader
                        .start(device, endpoint, stop_flag.clone(), move |buf| {
                            println!("Recebi pacote: {:?}", buf);

                            let emit_flow: Vec<EmitCommand> = translator.conv(&buf);

                            println!("Pacote Convertido: {:?}", emit_flow);

                            for emit in emit_flow {
                                match emit {
                                    EmitCommand::Pen {
                                        x,
                                        y,
                                        pressure,
                                        touch,
                                    } => {
                                        if let Err(e) = vpen_clone.emit(x, y, pressure, touch) {
                                            eprintln!("Erro emitindo evento: {e}");
                                        }

                                        let encoded = serde_json::to_string(&emit).unwrap();
                                        let _ = tx_socket.send(format!("{}\n", encoded).into_bytes());

                                    }
                                    EmitCommand::Btn { key, pressed } => {
                                        if let Err(e) = vbtn_clone.emit(Key::new(key as u16), pressed) {
                                            eprintln!("Erro emitindo botão: {e}");
                                        }
                                        let encoded = serde_json::to_string(&emit).unwrap();
                                        let _ = tx_socket.send(format!("{}\n", encoded).into_bytes());
                                    }
                                }
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
