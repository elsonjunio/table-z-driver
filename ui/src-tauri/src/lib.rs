use std::os::unix::net::UnixStream;
use std::io::{BufReader, BufRead, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use table_z_config::{Config, PenConfig, ActionsConfig, SettingsConfig};
use std::path::Path;
use tauri::{AppHandle, Emitter, Manager};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use std::env;
//use table_z_config::Config;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct PenStatus {
    x: i32,
    y: i32,
    pressure: i32,
    touch: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmitCommand {
    Pen {
        x: i32,
        y: i32,
        pressure: i32,
        touch: bool,
    },
    Btn {
        key: i32,
        pressed: bool,
        index: usize,
    },
}


// Mantemos a conexão compartilhada
struct DriverConnection {
    stream: Mutex<UnixStream>,
}


/// Caminho preferencial do usuário ($HOME/table_z_utils.yaml)
fn user_config_path() -> PathBuf {
    let home = env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join("table_z_utils.yaml")
}

/// Caminho global (/etc/table_z_utils.yaml)
fn system_config_path() -> PathBuf {
    PathBuf::from("/etc/table_z_utils.yaml")
}


/// Carrega configuração, com fallback e cópia para $HOME
fn load_or_create_config() -> Result<Config, String> {
    let user_path = user_config_path();
    let system_path = system_config_path();

    // 1. tenta no $HOME
    if user_path.exists() {
        return Config::from_file(&user_path).map_err(|e| e.to_string());
    }

    // 2. tenta no /etc
    if system_path.exists() {
        let cfg = Config::from_file(&system_path).map_err(|e| e.to_string())?;

        // 3. tenta salvar no $HOME para a próxima vez
        if let Ok(yaml) = serde_yaml::to_string(&cfg) {
            if let Err(e) = fs::write(&user_path, yaml) {
                eprintln!("Aviso: não consegui salvar config em {:?}: {}", user_path, e);
            }
        }

        return Ok(cfg);
    }

    Err("Nenhum arquivo de configuração encontrado".into())
}


#[tauri::command]
fn update_config(config: Config, state: tauri::State<Arc<DriverConnection>>) -> Result<(), String> {
    let mut stream = state.stream.lock().map_err(|_| "Lock error")?;
    let msg = serde_json::to_string(&config).map_err(|e| e.to_string())?;
    let user_path = user_config_path();

    //// 3. tenta salvar no $HOME para a próxima vez
    if let Ok(yaml) = serde_yaml::to_string(&config) {
        if let Err(e) = fs::write(&user_path, yaml) {
            eprintln!("Aviso: não consegui salvar config em {:?}: {}", user_path, e);
        }
    }

    stream.write_all(msg.as_bytes()).map_err(|e| e.to_string())?;
    stream.write_all(b"\n").map_err(|e| e.to_string())?;
    Ok(())
}

//#[tauri::command]
//fn get_config() -> Result<Config, String> {
//    let cfg = Config::from_file(Path::new("/etc/table_z_utils.yaml"))
//        .map_err(|e| e.to_string())?;
//    Ok(cfg)
//}

#[tauri::command]
fn get_config() -> Result<Config, String> {
    load_or_create_config()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
    .setup(|app| {

        let app_handle = app.handle();
        let socket_path = "/tmp/tablet.sock";

        // Tenta conectar no driver
        let stream = UnixStream::connect(socket_path)
            .expect("Não foi possível conectar ao driver");

        let driver = Arc::new(DriverConnection {
            stream: Mutex::new(stream.try_clone().unwrap()),
        });


        // 🔹 Carrega configuração (user ou /etc)
        match load_or_create_config() {
            Ok(cfg) => {
                // envia para o driver
                if let Ok(json) = serde_json::to_string(&cfg) {
                    if let Ok(mut s) = driver.stream.lock() {
                        if let Err(e) = s.write_all(json.as_bytes()) {
                            eprintln!("Falha ao enviar config inicial: {}", e);
                        }
                        let _ = s.write_all(b"\n");
                    }
                }
            }
            Err(e) => {
                eprintln!("Não foi possível carregar config: {}", e);
            }
        }


        let reader_stream = stream;
        let app_handle_clone = app_handle.clone();

        thread::spawn(move || {
            let reader = BufReader::new(reader_stream);

            for line in reader.lines() {
                if let Ok(cmd) = serde_json::from_str::<EmitCommand>(&line.unwrap()) {
                    println!("table: {:?}", cmd);
                    match cmd {
                        EmitCommand::Pen { x, y, pressure, touch } => {
                            //let status = PenStatus { x, y, pressure, touch };
                            let status = EmitCommand::Pen { x, y, pressure, touch };
                            let _ = app_handle_clone.emit("pen_event", status);
                        }
                        EmitCommand::Btn { key, pressed, index } => {
                            let status = EmitCommand::Btn { key, pressed, index };
                            let _ = app_handle_clone.emit("btn_event", status);
                        }
                    }
                }
            }
        });

        app.manage(driver);

        Ok(())

    })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![update_config, get_config])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
