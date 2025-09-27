use std::sync::{Arc, Mutex};
use bincode::{Encode, Decode};
use serde::{Serialize, Deserialize};

use std::collections::HashMap;
use evdev::Key;



#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize)]
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

pub trait Translator: Send + Sync {
    /// Traduz um pacote da USB em uma lista de comandos
    fn conv(&self, buf: &Vec<u8>) -> Vec<EmitCommand>;
}

fn key_from_str(name: &str) -> Option<Key> {
    let map: HashMap<&'static str, Key> = [
        ("BTN_TOOL_PEN", Key::BTN_TOOL_PEN),
        ("BTN_STYLUS", Key::BTN_STYLUS),
        ("BTN_TOUCH", Key::BTN_TOUCH),
        ("KEY_LEFTCTRL", Key::KEY_LEFTCTRL),
        ("KEY_Z", Key::KEY_Z),
        ("KEY_A", Key::KEY_A),
        ("KEY_C", Key::KEY_C),
        ("KEY_D", Key::KEY_D),
        // se precisar de mais, só adicionar
    ]
    .into_iter()
    .collect();

    map.get(name).copied()
}
