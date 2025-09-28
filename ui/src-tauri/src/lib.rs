use std::os::unix::net::UnixStream;
use std::io::{BufReader, BufRead, Write};
use std::sync::{Arc, Mutex};
use std::thread;

use tauri::{AppHandle, Emitter, Manager};
use serde::{Deserialize, Serialize};
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
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct TabletConfig {
    invertX: bool,
    invertY: bool,
}

// Mantemos a conexão compartilhada
struct DriverConnection {
    stream: Mutex<UnixStream>,
}

#[tauri::command]
fn update_config(config: TabletConfig, state: tauri::State<Arc<DriverConnection>>) -> Result<(), String> {
    let mut stream = state.stream.lock().map_err(|_| "Lock error")?;
    let msg = serde_json::to_string(&config).map_err(|e| e.to_string())?;
    stream.write_all(msg.as_bytes()).map_err(|e| e.to_string())?;
    stream.write_all(b"\n").map_err(|e| e.to_string())?;
    Ok(())
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

        let reader_stream = stream;
        let app_handle_clone = app_handle.clone();

        thread::spawn(move || {
            let reader = BufReader::new(reader_stream);

            for line in reader.lines() {
                //if let Ok(line) = line {
                //    println!("table: {:?}", line);
                //    if let Ok(status) = serde_json::from_str::<TabletStatus>(&line) {
                //        let _ = app_handle_clone.emit("tablet_event", status);
                //    }
                //}
                if let Ok(cmd) = serde_json::from_str::<EmitCommand>(&line.unwrap()) {
                    println!("table: {:?}", cmd);
                    match cmd {
                        EmitCommand::Pen { x, y, pressure, touch } => {
                            //let status = PenStatus { x, y, pressure, touch };
                            let status = EmitCommand::Pen { x, y, pressure, touch };
                            let _ = app_handle_clone.emit("pen_event", status);
                        }
                        EmitCommand::Btn { key, pressed } => {
                            let status = EmitCommand::Btn { key, pressed };
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
        .invoke_handler(tauri::generate_handler![update_config])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
