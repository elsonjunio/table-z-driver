use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::error::Error;

/// Config principal
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub xinput_name: String,
    pub vendor_id: u16,
    pub product_id: u16,
    pub interface: u8,
    pub pen: PenConfig,
    pub actions: ActionsConfig,
    pub settings: SettingsConfig,
}

/// Configuração da caneta
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PenConfig {
    pub max_x: u32,
    pub max_y: u32,
    pub max_pressure: u32,
    pub resolution_x: u32,
    pub resolution_y: u32,
}

/// Configuração de ações
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ActionsConfig {
    pub pen: String,
    pub stylus: String,
    pub pen_touch: String,
    pub tablet_buttons: Vec<String>,
}

/// Configurações gerais
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SettingsConfig {
    pub swap_axis: bool,
    pub swap_direction_x: bool,
    pub swap_direction_y: bool,
}

impl Config {
    /// Lê o YAML e deserializa para Config
    pub fn from_file(path: &Path) -> Result<Config, Box<dyn Error>> {
        let content = fs::read_to_string(path)?;
        let cfg: Config = serde_yaml::from_str(&content)?;
        Ok(cfg)
    }
}
