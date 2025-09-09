use std::sync::{Arc, Mutex};


#[derive(Debug, Clone)]
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
