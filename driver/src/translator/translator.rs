use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use table_z_config::Config;

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
        index: usize,
    },
}

pub trait Translator: Send + Sync {
    /// Traduz um pacote da USB em uma lista de comandos
    fn conv(&self, buf: &Vec<u8>) -> Vec<EmitCommand>;
    fn update_from_config(&mut self, cfg: &Config);
}
