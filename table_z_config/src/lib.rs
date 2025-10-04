use serde::{Deserialize, Serialize};
use std::{fs, path::Path};
use std::error::Error;

/// Representa a configuração principal do dispositivo/tablet.
///
/// Este arquivo é carregado de um YAML e define os parâmetros de hardware,
/// ações mapeadas e ajustes de comportamento.
///
/// Exemplo de YAML:
/// ```yaml
/// xinput_name: "Tablet M100"
/// vendor_id: 1234
/// product_id: 5678
/// interface: 1
/// pen:
///   max_x: 32767
///   max_y: 32767
///   max_pressure: 8192
///   resolution_x: 100
///   resolution_y: 100
/// actions:
///   pen: "BTN_LEFT"
///   stylus: "BTN_RIGHT"
///   pen_touch: "BTN_TOUCH"
///   tablet_buttons:
///     - "KEY_A"
///     - "KEY_B"
/// settings:
///   swap_axis: false
///   swap_direction_x: false
///   swap_direction_y: false
/// ```
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    /// Nome do dispositivo virtual (XInput) a ser criado.
    pub xinput_name: String,

    /// IDs USB do dispositivo.
    pub vendor_id: u16,
    pub product_id: u16,

    /// Interface (endpoint) usada para leitura USB.
    pub interface: u8,

    /// Configurações físicas da caneta.
    pub pen: PenConfig,

    /// Mapeamentos de botões e ações.
    pub actions: ActionsConfig,

    /// Ajustes de eixos e transformações.
    pub settings: SettingsConfig,
}

/// Define os parâmetros físicos da caneta (limites e resolução).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PenConfig {
    pub max_x: u32,
    pub max_y: u32,
    pub max_pressure: u32,
    pub resolution_x: u32,
    pub resolution_y: u32,
}

/// Define o mapeamento das ações e botões configuráveis.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ActionsConfig {
    /// Código de tecla para o clique da caneta (ex: "BTN_LEFT").
    pub pen: String,

    /// Código de tecla para o botão lateral da caneta.
    pub stylus: String,

    /// Código de tecla para o toque da ponta da caneta.
    pub pen_touch: String,

    /// Lista de combinações de botões físicos no tablet.
    pub tablet_buttons: Vec<String>,
}

/// Define ajustes de comportamento da leitura do dispositivo.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SettingsConfig {
    /// Se `true`, inverte os eixos X e Y.
    pub swap_axis: bool,

    /// Inverte a direção do eixo X.
    pub swap_direction_x: bool,

    /// Inverte a direção do eixo Y.
    pub swap_direction_y: bool,
}

impl Config {
    /// Carrega e parseia o arquivo YAML de configuração.
    ///
    /// # Parâmetros
    /// - `path`: Caminho para o arquivo YAML (ex: `/etc/table_z_utils.yaml`)
    ///
    /// # Retorno
    /// - `Ok(Config)` se o arquivo for lido e interpretado corretamente.
    /// - `Err` se o arquivo não existir, estiver ilegível ou mal formatado.
    ///
    /// # Exemplo
    /// ```
    /// let cfg = Config::from_file(Path::new("/etc/table_z_utils.yaml")).unwrap();
    /// println!("Nome do dispositivo: {}", cfg.xinput_name);
    /// ```
    pub fn from_file(path: &Path) -> Result<Self, Box<dyn Error + Send + Sync>> {
        // Verifica se o arquivo existe antes de tentar ler.
        if !path.exists() {
            return Err(format!("Arquivo de configuração não encontrado: {}", path.display()).into());
        }

        // Lê o conteúdo do arquivo YAML como string.
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Erro ao ler arquivo {}: {e}", path.display()))?;

        // Faz o parse YAML -> struct Config.
        let cfg: Config = serde_yaml::from_str(&content)
            .map_err(|e| format!("Erro ao interpretar YAML: {e}"))?;

        Ok(cfg)
    }
}
