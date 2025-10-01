use std::{str::FromStr, sync::{Arc, Mutex}};
use std::collections::HashSet;

use evdev::Key;

use crate::{
    translator::translator::{EmitCommand, Translator},
};

use table_z_config::Config;

pub struct TabletM100Translator {
    // Propriedades do hardware
    pen_max_x: u32,
    pen_max_y: u32,
    pen_max_pressure: u32,
    pen_resolution_x: u32,
    pen_resolution_y: u32,

    // Ações (botões e toques)
    action_pen: Key,
    action_stylus: Key,
    action_pen_touch: Key,
    pub action_tablet_buttons: Vec<Vec<Key>>,

    // Flags de transformação
    swap_axis: bool,
    swap_direction_x: bool,
    swap_direction_y: bool,

    pressed_keys: Mutex<HashSet<usize>>, // índice de action_tablet_buttons
}

impl TabletM100Translator {
    pub fn new(cfg: Arc<Mutex<Config>>) -> Self {
        let cfg_guard = cfg.lock().unwrap();

        // Converte strings em Keys
        let action_pen = Key::from_str(&cfg_guard.actions.pen).expect("Chave inválida para pen");
        let action_stylus =
            Key::from_str(&cfg_guard.actions.stylus).expect("Chave inválida para stylus");
        let action_pen_touch =
            Key::from_str(&cfg_guard.actions.pen_touch).expect("Chave inválida para pen_touch");

        let action_tablet_buttons = cfg_guard
            .actions
            .tablet_buttons
            .iter()
            .map(|combo_str| {
                combo_str
                    .split('+')
                    .map(|s| {
                        Key::from_str(s.trim()).expect("Chave inválida")
                    })
                    .collect::<Vec<Key>>()
            })
            .collect::<Vec<Vec<Key>>>();

        println!("action_tablet_buttons: {:?}", action_tablet_buttons);

        Self {
            // Propriedades escalares
            pen_max_x: cfg_guard.pen.max_x,
            pen_max_y: cfg_guard.pen.max_y,
            pen_max_pressure: cfg_guard.pen.max_pressure,
            pen_resolution_x: cfg_guard.pen.resolution_x,
            pen_resolution_y: cfg_guard.pen.resolution_y,

            // Ações
            action_pen,
            action_stylus,
            action_pen_touch,
            action_tablet_buttons,

            // Flags
            swap_axis: cfg_guard.settings.swap_axis,
            swap_direction_x: cfg_guard.settings.swap_direction_x,
            swap_direction_y: cfg_guard.settings.swap_direction_y,
            pressed_keys: Mutex::new(HashSet::new()),
        }
    }
}

//impl Translator for TabletM100Translator {
//    fn conv(&self, buf: &Vec<u8>) -> Vec<EmitCommand> {
//        let mut out = Vec::new();
//
//        if buf.len() >= 8 && (buf[1] == 192 || buf[1] == 193) {
//
//
//            let raw_x = (buf[5] as i32 * 255 + buf[4] as i32);
//            let raw_y = (buf[3] as i32 * 255 + buf[2] as i32);
//            let raw_pressure = (buf[7] as i32 * 255 + buf[6] as i32);
//
//            let mut x = raw_x;
//            let mut y = raw_y;
//            let mut pressure = raw_pressure;
//
//            // Aplica swap de eixo
//            if self.swap_axis {
//                std::mem::swap(&mut x, &mut y);
//            }
//
//            // Aplica inversão de direção
//            if self.swap_direction_x {
//                x = self.pen_max_x as i32 - x;
//            }
//
//            if self.swap_direction_y {
//                y = self.pen_max_y as i32 - y;
//            }
//
//            let touching = buf[1] != 192;
//
//            out.push(EmitCommand::Pen {
//                x,
//                y,
//                pressure,
//                touch: touching,
//            });
//        }
//        else if buf.len() >= 8 && buf[0] == 2 {
//    
//            let mut key_index: Option<usize> = None;
//
//            if buf[1] == 1 && buf[3] == 28 {
//                out.push(EmitCommand::Btn {
//                    key: Key::BTN_STYLUS.code() as i32,
//                    pressed: true
//                });
//            }
//
//            if buf[1] == 1 && buf[3] == 29 {
//                out.push(EmitCommand::Btn {
//                    key: Key::BTN_STYLUS2.code() as i32,
//                    pressed: true
//                });
//            }
//
//        }
//
//        out
//    }
//}

impl Translator for TabletM100Translator {

    fn update_from_config(&mut self, cfg: &Config) {
        use evdev::Key;
        use std::str::FromStr;

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

    fn conv(&self, buf: &Vec<u8>) -> Vec<EmitCommand> {
        let mut out = Vec::new();

        // --- Movimento da caneta ---
        if buf.len() >= 8 && (buf[1] == 192 || buf[1] == 193) {
            let raw_x = (buf[5] as i32 * 255 + buf[4] as i32);
            let raw_y = (buf[3] as i32 * 255 + buf[2] as i32);
            let raw_pressure = (buf[7] as i32 * 255 + buf[6] as i32);

            let mut x = raw_x;
            let mut y = raw_y;
            let mut pressure = raw_pressure;

            if self.swap_axis { std::mem::swap(&mut x, &mut y); }
            if self.swap_direction_x { x = self.pen_max_x as i32 - x; }
            if self.swap_direction_y { y = self.pen_max_y as i32 - y; }

            out.push(EmitCommand::Pen { x, y, pressure, touch: buf[1] != 192 });
        }
        // --- Botões ---
        else if buf.len() >= 8 && buf[0] == 2 {
            let mut pressed_keys = self.pressed_keys.lock().unwrap();
            let mut current_keys = HashSet::new();

            // mapeamento manual do seu dispositivo
            let button_mapping: Vec<(u8, u8, usize)> = vec![
                (1, 28, 5000), // BTN_STYLUS
                (1, 29, 5001), // BTN_STYLUS2
                (1, 86, 0), // btn 1    
                (1, 87, 1), // btn 2
                (0, 47, 2), // btn 3
                (0, 48, 3), // btn 4
                (0, 43, 4), // btn 5
                (0, 44, 5), // btn 6
                (1, 0, 6), // btn 7
                (4, 0, 7), // btn 8
            ];

            for (b1, b3, idx) in button_mapping.iter() {
                if buf[1] == *b1 && buf[3] == *b3 {
                    current_keys.insert(*idx);
                }
            }
            for idx in current_keys.difference(&pressed_keys) {
                if *idx >= 5000 {
                    // Aqui decide qual Key corresponde ao idx "virtual"
                    let key = match *idx {
                        5000 => Key::BTN_STYLUS,
                        5001 => Key::BTN_STYLUS2,
                        _ => continue,
                    };
                    out.push(EmitCommand::Btn { key: key.code() as i32, pressed: true, index: idx.clone() });
                } else if let Some(keys) = self.action_tablet_buttons.get(*idx) {
                    for k in keys {
                        out.push(EmitCommand::Btn { key: k.code() as i32, pressed: true, index: idx.clone() });
                    }
                }
            }
            
            // Mesma lógica para release:
            for idx in pressed_keys.difference(&current_keys) {
                if *idx >= 5000 {
                    let key = match *idx {
                        5000 => Key::BTN_STYLUS,
                        5001 => Key::BTN_STYLUS2,
                        _ => continue,
                    };
                    out.push(EmitCommand::Btn { key: key.code() as i32, pressed: false, index: idx.clone() });
                } else if let Some(keys) = self.action_tablet_buttons.get(*idx) {
                    for k in keys {
                        out.push(EmitCommand::Btn { key: k.code() as i32, pressed: false, index: idx.clone() });
                    }
                }
            }
            // Atualiza estado
            *pressed_keys = current_keys;
        }

        out
    }
}
