use std::collections::HashSet;
use std::{
    str::FromStr,
    sync::{Arc, Mutex},
};

use evdev::Key;
use crate::translator::translator::{EmitCommand, Translator};
use table_z_config::Config;

/// Tradutor responsável por interpretar os pacotes de dados de um tablet modelo M100
/// e convertê-los em comandos lógicos de entrada (`EmitCommand`).
///
/// Este componente é responsável por:
/// - Interpretar pacotes USB do tablet
/// - Converter valores brutos de coordenadas e pressão em eventos de caneta
/// - Mapear botões físicos do tablet para combinações configuráveis de teclas
/// - Gerenciar o estado de teclas pressionadas para emitir eventos corretos
pub struct TabletM100Translator {
    // --- Propriedades do hardware ---
    /// Valor máximo do eixo X
    pen_max_x: u32,
    /// Valor máximo do eixo Y
    pen_max_y: u32,
    /// Pressão máxima reconhecida pela caneta
    pen_max_pressure: u32,
    /// Resolução em DPI do eixo X
    pen_resolution_x: u32,
    /// Resolução em DPI do eixo Y
    pen_resolution_y: u32,

    // --- Ações configuráveis ---
    /// Tecla associada ao clique da caneta
    action_pen: Key,
    /// Tecla associada ao botão lateral (stylus)
    action_stylus: Key,
    /// Tecla associada ao toque da caneta na superfície
    action_pen_touch: Key,
    /// Lista de combinações de teclas para os botões físicos do tablet
    pub action_tablet_buttons: Vec<Vec<Key>>,

    // --- Flags de transformação ---
    /// Inverte eixos X e Y
    swap_axis: bool,
    /// Inverte a direção do eixo X
    swap_direction_x: bool,
    /// Inverte a direção do eixo Y
    swap_direction_y: bool,

    /// Conjunto de índices de botões atualmente pressionados
    pressed_keys: Mutex<HashSet<usize>>,
}

impl TabletM100Translator {
    /// Cria uma nova instância do tradutor a partir de uma configuração compartilhada (`Arc<Mutex<Config>>`)
    pub fn new(cfg: Arc<Mutex<Config>>) -> Self {
        let cfg_guard = cfg.lock().unwrap();

        // --- Conversão de strings para `Key` ---
        let action_pen = Key::from_str(&cfg_guard.actions.pen)
            .expect("Chave inválida para pen");
        let action_stylus = Key::from_str(&cfg_guard.actions.stylus)
            .expect("Chave inválida para stylus");
        let action_pen_touch = Key::from_str(&cfg_guard.actions.pen_touch)
            .expect("Chave inválida para pen_touch");

        // Converte botões configurados como combinações (ex: "Ctrl+Z")
        let action_tablet_buttons = cfg_guard
            .actions
            .tablet_buttons
            .iter()
            .map(|combo_str| {
                combo_str
                    .split('+')
                    .map(|s| Key::from_str(s.trim()).expect("Chave inválida"))
                    .collect::<Vec<Key>>()
            })
            .collect::<Vec<Vec<Key>>>();

        println!("action_tablet_buttons: {:?}", action_tablet_buttons);

        Self {
            pen_max_x: cfg_guard.pen.max_x,
            pen_max_y: cfg_guard.pen.max_y,
            pen_max_pressure: cfg_guard.pen.max_pressure,
            pen_resolution_x: cfg_guard.pen.resolution_x,
            pen_resolution_y: cfg_guard.pen.resolution_y,
            action_pen,
            action_stylus,
            action_pen_touch,
            action_tablet_buttons,
            swap_axis: cfg_guard.settings.swap_axis,
            swap_direction_x: cfg_guard.settings.swap_direction_x,
            swap_direction_y: cfg_guard.settings.swap_direction_y,
            pressed_keys: Mutex::new(HashSet::new()),
        }
    }
}

impl Translator for TabletM100Translator {
    /// Atualiza o tradutor a partir de uma nova configuração.
    fn update_from_config(&mut self, cfg: &Config) {
        self.pen_max_x = cfg.pen.max_x;
        self.pen_max_y = cfg.pen.max_y;
        self.pen_max_pressure = cfg.pen.max_pressure;
        self.pen_resolution_x = cfg.pen.resolution_x;
        self.pen_resolution_y = cfg.pen.resolution_y;

        self.action_pen = Key::from_str(&cfg.actions.pen).unwrap();
        self.action_stylus = Key::from_str(&cfg.actions.stylus).unwrap();
        self.action_pen_touch = Key::from_str(&cfg.actions.pen_touch).unwrap();

        self.action_tablet_buttons = cfg
            .actions
            .tablet_buttons
            .iter()
            .map(|combo| {
                combo
                    .split('+')
                    .filter_map(|k| Key::from_str(k).ok())
                    .collect::<Vec<Key>>()
            })
            .collect();

        self.swap_axis = cfg.settings.swap_axis;
        self.swap_direction_x = cfg.settings.swap_direction_x;
        self.swap_direction_y = cfg.settings.swap_direction_y;
    }

    /// Converte um buffer de bytes do dispositivo USB em uma lista de comandos interpretados.
    ///
    /// - Pacotes com `buf[1] == 192 ou 193` representam movimento da caneta
    /// - Pacotes com `buf[0] == 2` representam botões físicos
    fn conv(&self, buf: &Vec<u8>) -> Vec<EmitCommand> {
        let mut out = Vec::new();

        // --- Movimento da caneta ---
        if buf.len() >= 8 && (buf[1] == 192 || buf[1] == 193) {
            let raw_x = buf[5] as i32 * 255 + buf[4] as i32;
            let raw_y = buf[3] as i32 * 255 + buf[2] as i32;
            let raw_pressure = buf[7] as i32 * 255 + buf[6] as i32;

            let mut x = raw_x;
            let mut y = raw_y;
            let pressure = raw_pressure;

            // Aplica transformações configuradas
            if self.swap_axis {
                std::mem::swap(&mut x, &mut y);
            }
            if self.swap_direction_x {
                x = self.pen_max_x as i32 - x;
            }
            if self.swap_direction_y {
                y = self.pen_max_y as i32 - y;
            }

            out.push(EmitCommand::Pen {
                x,
                y,
                pressure,
                touch: buf[1] != 192,
            });
        }

        // --- Botões ---
        else if buf.len() >= 8 && buf[0] == 2 {
            let mut pressed_keys = self.pressed_keys.lock().unwrap();
            let mut current_keys = HashSet::new();

            // Mapeamento estático do dispositivo
            let button_mapping: Vec<(u8, u8, usize)> = vec![
                (1, 28, 5000), // BTN_STYLUS
                (1, 29, 5001), // BTN_STYLUS2
                (1, 86, 0),    // Botão 1
                (1, 87, 1),    // Botão 2
                (0, 47, 2),    // Botão 3
                (0, 48, 3),    // Botão 4
                (0, 43, 4),    // Botão 5
                (0, 44, 5),    // Botão 6
                (1, 0, 6),     // Botão 7
                (4, 0, 7),     // Botão 8
            ];

            // Identifica quais botões estão pressionados neste pacote
            for (b1, b3, idx) in button_mapping.iter() {
                if buf[1] == *b1 && buf[3] == *b3 {
                    current_keys.insert(*idx);
                }
            }

            // Detecta botões pressionados
            for idx in current_keys.difference(&pressed_keys) {
                if *idx >= 5000 {
                    let key = match *idx {
                        5000 => Key::BTN_STYLUS,
                        5001 => Key::BTN_STYLUS2,
                        _ => continue,
                    };
                    out.push(EmitCommand::Btn {
                        key: key.code() as i32,
                        pressed: true,
                        index: *idx,
                    });
                } else if let Some(keys) = self.action_tablet_buttons.get(*idx) {
                    for k in keys {
                        out.push(EmitCommand::Btn {
                            key: k.code() as i32,
                            pressed: true,
                            index: *idx,
                        });
                    }
                }
            }

            // Detecta botões liberados
            for idx in pressed_keys.difference(&current_keys) {
                if *idx >= 5000 {
                    let key = match *idx {
                        5000 => Key::BTN_STYLUS,
                        5001 => Key::BTN_STYLUS2,
                        _ => continue,
                    };
                    out.push(EmitCommand::Btn {
                        key: key.code() as i32,
                        pressed: false,
                        index: *idx,
                    });
                } else if let Some(keys) = self.action_tablet_buttons.get(*idx) {
                    for k in keys {
                        out.push(EmitCommand::Btn {
                            key: k.code() as i32,
                            pressed: false,
                            index: *idx,
                        });
                    }
                }
            }

            // Atualiza estado
            *pressed_keys = current_keys;
        }

        out
    }
}
