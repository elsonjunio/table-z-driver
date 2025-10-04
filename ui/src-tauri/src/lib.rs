use std::os::unix::net::UnixStream;
use std::io::{BufReader, BufRead, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use table_z_config::{Config};
use std::env;

use tauri::{
    AppHandle, Manager, Emitter, Wry,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};

use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

/// Representa o estado da caneta no momento
#[derive(Debug, Serialize, Deserialize, Clone)]
struct PenStatus {
    pub x: i32,
    pub y: i32,
    pub pressure: i32,
    pub touch: bool,
}

/// Comandos emitidos pelo driver (caneta ou botões)
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

/// Estrutura para manter a conexão com o driver de forma compartilhada
struct DriverConnection {
    pub stream: Mutex<UnixStream>,
}

/// Retorna o caminho preferencial do arquivo de configuração do usuário (~/$HOME)
fn user_config_path() -> PathBuf {
    let home = env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join("table_z_utils.yaml")
}

/// Retorna o caminho global de configuração (/etc/table_z_utils.yaml)
fn system_config_path() -> PathBuf {
    PathBuf::from("/etc/table_z_utils.yaml")
}

/// Carrega a configuração do usuário ou do sistema. Se só existir a do sistema,
/// copia para $HOME para uso futuro.
///
/// # Erros
/// Retorna `Err` se nenhum arquivo de configuração existir ou não puder ser lido.
fn load_or_create_config() -> Result<Config, String> {
    let user_path = user_config_path();
    let system_path = system_config_path();

    // 1. Tenta no $HOME
    if user_path.exists() {
        return Config::from_file(&user_path).map_err(|e| e.to_string());
    }

    // 2. Tenta no /etc
    if system_path.exists() {
        let cfg = Config::from_file(&system_path).map_err(|e| e.to_string())?;

        // 3. Copia para $HOME para a próxima vez
        if let Ok(yaml) = serde_yaml::to_string(&cfg) {
            if let Err(e) = fs::write(&user_path, yaml) {
                eprintln!("Aviso: não consegui salvar config em {:?}: {}", user_path, e);
            }
        }

        return Ok(cfg);
    }

    Err("Nenhum arquivo de configuração encontrado".into())
}

/// Atualiza a configuração e envia para o driver via socket
#[tauri::command]
fn update_config(
    config: Config,
    state: tauri::State<Arc<DriverConnection>>,
) -> Result<(), String> {
    let mut stream = state.stream.lock().map_err(|_| "Lock error")?;
    let msg = serde_json::to_string(&config).map_err(|e| e.to_string())?;
    let user_path = user_config_path();

    // Tenta salvar no $HOME
    if let Ok(yaml) = serde_yaml::to_string(&config) {
        if let Err(e) = fs::write(&user_path, yaml) {
            eprintln!("Aviso: não consegui salvar config em {:?}: {}", user_path, e);
        }
    }

    stream.write_all(msg.as_bytes()).map_err(|e| e.to_string())?;
    stream.write_all(b"\n").map_err(|e| e.to_string())?;
    Ok(())
}

/// Recupera a configuração carregada do usuário ou sistema
#[tauri::command]
fn get_config() -> Result<Config, String> {
    load_or_create_config()
}

/// Cria o menu do tray com opções de mostrar, ocultar e sair
fn create_tray_menu(app_handle: &AppHandle) -> Result<Menu<Wry>, Box<dyn std::error::Error>> {
    let show = MenuItem::with_id(app_handle, "show", "Mostrar", true, None::<&str>)?;
    let hide = MenuItem::with_id(app_handle, "hide", "Ocultar", true, None::<&str>)?;
    let quit = MenuItem::with_id(app_handle, "quit", "Sair", true, None::<&str>)?;

    let menu = Menu::with_items(app_handle, &[
        &show,
        &hide,
        &PredefinedMenuItem::separator(app_handle)?,
        &quit,
    ])?;

    Ok(menu)
}

/// Ponto de entrada principal da aplicação Tauri
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.handle();
            let socket_path = "/tmp/tablet.sock";

            // Conecta ao driver via Unix socket
            let stream = UnixStream::connect(socket_path)
                .expect("Não foi possível conectar ao driver");

            let driver = Arc::new(DriverConnection {
                stream: Mutex::new(stream.try_clone().unwrap()),
            });

            // 🔹 Carrega configuração
            if let Ok(cfg) = load_or_create_config() {
                if let Ok(json) = serde_json::to_string(&cfg) {
                    if let Ok(mut s) = driver.stream.lock() {
                        if let Err(e) = s.write_all(json.as_bytes()) {
                            eprintln!("Falha ao enviar config inicial: {}", e);
                        }
                        let _ = s.write_all(b"\n");
                    }
                }
            }

            let reader_stream = stream;
            let app_handle_clone = app_handle.clone();

            // Thread para receber eventos do driver
            thread::spawn(move || {
                let reader = BufReader::new(reader_stream);

                for line in reader.lines() {
                    if let Ok(cmd) = serde_json::from_str::<EmitCommand>(&line.unwrap()) {
                        match cmd {
                            EmitCommand::Pen { x, y, pressure, touch } => {
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

            let window = app_handle.get_webview_window("main").unwrap();
            let _ = window.hide();

            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![update_config, get_config])
        .on_page_load(|_window, _| {})
        .build(tauri::generate_context!())
        .expect("erro ao iniciar Tauri")
        .run(|app_handle, event| {
            match event {
                tauri::RunEvent::Ready => {
                    // Cria menu e ícone do tray
                    if let Ok(tray_menu) = create_tray_menu(app_handle) {
                        let _ = TrayIconBuilder::new()
                            .icon(app_handle.default_window_icon().unwrap().clone())
                            .menu(&tray_menu)
                            .on_menu_event(move |app_handle, event| {
                                match event.id.as_ref() {
                                    "show" => {
                                        if let Some(window) = app_handle.get_webview_window("main") {
                                            let _ = window.show();
                                            let _ = window.set_focus();
                                        }
                                    }
                                    "hide" => {
                                        if let Some(window) = app_handle.get_webview_window("main") {
                                            let _ = window.hide();
                                        }
                                    }
                                    "quit" => std::process::exit(0),
                                    _ => {}
                                }
                            })
                            .on_tray_icon_event(|tray, event| {
                                if let TrayIconEvent::Click { button, button_state, .. } = event {
                                    if button == MouseButton::Left && button_state == MouseButtonState::Up {
                                        if let Some(window) = tray.app_handle().get_webview_window("main") {
                                            if let Ok(visible) = window.is_visible() {
                                                if visible {
                                                    let _ = window.hide();
                                                } else {
                                                    let _ = window.show();
                                                    let _ = window.set_focus();
                                                }
                                            }
                                        }
                                    }
                                }
                            })
                            .build(app_handle);
                    }
                }

                tauri::RunEvent::ExitRequested { api, .. } => {
                    api.prevent_exit();
                }

                tauri::RunEvent::WindowEvent { label, event, .. } => {
                    if label == "main" {
                        if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                            // impede fechamento e apenas oculta
                            api.prevent_close();
                            if let Some(window) = app_handle.get_webview_window("main") {
                                let _ = window.hide();
                            }
                        }
                    }
                }

                _ => {}
            }
        });
}
