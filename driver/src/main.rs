//! Ponto de entrada do sistema de emulação de tablet/tablet PC via USB.
//!
//! Este módulo integra:
//! - Detecção automática (hotplug) de dispositivos USB compatíveis;
//! - Leitura contínua dos pacotes HID via `USBReader`;
//! - Tradução dos pacotes em comandos (`Translator` → `EmitCommand`);
//! - Emulação de dispositivos virtuais (`VPen`, `VBtn`) usando `evdev`;
//! - Comunicação via socket UNIX para controle e atualização de configuração.

mod com;
use std::error::Error;
mod hotplug;
mod reader;
mod translator;
mod virtual_device;

use evdev::Key;
use std::str::FromStr;
use std::path::Path;
use std::sync::{
    Arc, Mutex, OnceLock,
    atomic::{AtomicBool, Ordering},
};
use serde_json;
use anyhow::Result;

use crate::{
    com::socket::SocketServer,
    hotplug::{HotPlugHandler, hotplug::CustomHotplugEvent},
    reader::USBReader,
    translator::{
        tablet_m100_translator::TabletM100Translator,
        translator::{EmitCommand, Translator},
    },
    virtual_device::{VBtn, VPen},
};

use table_z_config::Config;

/// Sinal global usado para parar a thread de leitura USB ao desconectar o dispositivo.
static STOP_FLAG: OnceLock<Mutex<Option<Arc<AtomicBool>>>> = OnceLock::new();

/// Inicializa a estrutura global de controle (OnceLock).
fn init_globals() {
    STOP_FLAG.get_or_init(|| Mutex::new(None));
}

/// Retorna uma lista de todas as teclas válidas (0x000–0x2FF).
fn all_keys() -> Vec<Key> {
    (0..=0x2FFu16).map(Key::new).collect()
}

/// Função principal — inicializa o sistema, carrega a configuração e aguarda eventos de hotplug.
fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Inicializa globals
    init_globals();

    // Cria o servidor de socket UNIX para comunicação externa
    let socket_server = SocketServer::new("/tmp/tablet.sock");
    let tx_socket = socket_server.sender();

    // Carrega configuração principal
    let path = Path::new("/etc/table_z_utils.yaml");
    
    let cfg = Config::from_file(path)?;

    println!("Configuração carregada:\n{:#?}", cfg);

    // Cria tradutor com a configuração inicial
    let translator = Arc::new(Mutex::new(TabletM100Translator::new(Arc::new(Mutex::new(cfg.clone())))));

    // Inicializa sistema de hotplug USB
    HotPlugHandler::init({
        let cfg = cfg.clone();
        let translator = translator.clone();

        move |device, event| match event {
            CustomHotplugEvent::DeviceArrived => {
                // Dados de configuração
                let vendor_id: u16 = cfg.vendor_id;
                let product_id: u16 = cfg.product_id;
                let endpoint: u8 = cfg.interface;

                println!(
                    "Dispositivo conectado: {:?}",
                    device.device_descriptor().unwrap()
                );

                let desc = device.device_descriptor().unwrap();

                if desc.vendor_id() == vendor_id && desc.product_id() == product_id {
                    println!("Dispositivo compatível detectado!");

                    let usb_reader = USBReader::new().unwrap();
                    let stop_flag = Arc::new(AtomicBool::new(true));

                    // Define as teclas do dispositivo de caneta
                    let pen_keys: Vec<Key> = vec![
                        Key::from_str(&cfg.actions.pen).unwrap(),
                        Key::from_str(&cfg.actions.stylus).unwrap(),
                        Key::from_str(&cfg.actions.pen_touch).unwrap(),
                    ];

                    // Cria dispositivo virtual de caneta
                    let vpen = VPen::new(
                        cfg.pen.max_x as i32,
                        cfg.pen.max_y as i32,
                        cfg.pen.max_pressure as i32,
                        cfg.pen.resolution_x as i32,
                        cfg.pen.resolution_y as i32,
                        &pen_keys,
                        &cfg.xinput_name,
                    ).unwrap();

                    {
                        let mut guard = STOP_FLAG.get_or_init(|| Mutex::new(None)).lock().unwrap();
                        *guard = Some(stop_flag.clone());
                    }

                    // Cria dispositivo virtual de botões
                    let vbtn = VBtn::new(&all_keys(), &cfg.xinput_name).unwrap();

                    // Clones necessários para thread
                    let vbtn_clone = vbtn.clone();
                    let vpen_clone = vpen.clone();
                    let tx_socket = tx_socket.clone();
                    let translator = translator.clone();

                    // Inicia leitura contínua do USB
                    usb_reader.start(device, endpoint, stop_flag.clone(), move |buf| {
                        let emit_flow: Vec<EmitCommand> = translator.lock().unwrap().conv(&buf);

                        for emit in emit_flow {
                            match emit {
                                EmitCommand::Pen { x, y, pressure, touch } => {
                                    if let Err(e) = vpen_clone.emit(x, y, pressure, touch) {
                                        eprintln!("Erro emitindo evento: {e}");
                                    }

                                    if let Ok(encoded) = serde_json::to_string(&emit) {
                                        let _ = tx_socket.send(format!("{}\n", encoded).into_bytes());
                                    }
                                }
                                EmitCommand::Btn { key, pressed, index: _ } => {
                                    if let Err(e) = vbtn_clone.emit(Key::new(key as u16), pressed) {
                                        eprintln!("Erro emitindo botão: {e}");
                                    }

                                    if let Ok(encoded) = serde_json::to_string(&emit) {
                                        let _ = tx_socket.send(format!("{}\n", encoded).into_bytes());
                                    }
                                }
                            }
                        }
                    }).unwrap();
                }
            }
            CustomHotplugEvent::DeviceLeft => {
                println!("Dispositivo desconectado.");

                // Interrompe thread de leitura USB
                if let Some(flag) = STOP_FLAG
                    .get_or_init(|| Mutex::new(None))
                    .lock()
                    .unwrap()
                    .take()
                {
                    flag.store(false, Ordering::SeqCst);
                    println!("Thread de leitura parada.");
                }
            }
        }
    });

    println!("Loop principal iniciado...");

    // Loop principal de escuta via socket
    let translator_for_loop = translator.clone();
    loop {
        if let Some(cmd) = socket_server.try_recv_command() {
            println!("Comando recebido via socket: {}", cmd);

            if let Ok(new_cfg) = serde_json::from_str::<Config>(&cmd) {
                println!("Atualizando configuração em tempo de execução...");
                let mut tr = translator_for_loop.lock().unwrap();
                tr.update_from_config(&new_cfg);
            }
        }

        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
