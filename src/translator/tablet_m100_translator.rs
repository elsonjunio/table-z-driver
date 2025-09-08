use std::sync::{Arc, Mutex};

use evdev::Key;

use crate::{
    config::Config,
    key_from_str,
    translator::translator::{EmitCommand, Translator},
};

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
    action_tablet_buttons: Vec<Vec<Key>>,

    // Flags de transformação
    swap_axis: bool,
    swap_direction_x: bool,
    swap_direction_y: bool,
}

impl TabletM100Translator {
    pub fn new(cfg: Arc<Mutex<Config>>) -> Self {
        let cfg_guard = cfg.lock().unwrap();

        // Converte strings em Keys
        let action_pen = key_from_str(&cfg_guard.actions.pen).expect("Chave inválida para pen");
        let action_stylus =
            key_from_str(&cfg_guard.actions.stylus).expect("Chave inválida para stylus");
        let action_pen_touch =
            key_from_str(&cfg_guard.actions.pen_touch).expect("Chave inválida para pen_touch");

        let action_tablet_buttons = cfg_guard
            .actions
            .tablet_buttons
            .iter()
            .map(|combo_str| {
                combo_str
                    .split('+')
                    .map(|s| {
                        key_from_str(s.trim()).unwrap_or_else(|| panic!("Chave inválida: {}", s))
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
        }
    }
}

impl Translator for TabletM100Translator {
    fn conv(&self, buf: &Vec<u8>) -> Vec<EmitCommand> {
        let mut out = Vec::new();

        if buf.len() >= 8 && (buf[1] == 192 || buf[1] == 193) {


            let raw_x = (buf[5] as i32 * 255 + buf[4] as i32);
            let raw_y = (buf[3] as i32 * 255 + buf[2] as i32);
            let raw_pressure = (buf[7] as i32 * 255 + buf[6] as i32);

            let mut x = raw_x;
            let mut y = raw_y;
            let mut pressure = raw_pressure;

            //            // Escala para o range configurado
            //            let mut x = (raw_x as f32 / 65535.0 * self.pen_max_x as f32) as i32;
            //            let mut y = (raw_y as f32 / 65535.0 * self.pen_max_y as f32) as i32;
            //            let mut pressure =
            //                (raw_pressure as f32 / 65535.0 * self.pen_max_pressure as f32) as i32;

            // Aplica swap de eixo
            if self.swap_axis {
                std::mem::swap(&mut x, &mut y);
            }

            // Aplica inversão de direção
            if self.swap_direction_x {
                x = self.pen_max_x as i32 - x;
            }

            if self.swap_direction_y {
                y = self.pen_max_y as i32 - y;
            }

            let touching = buf[1] != 192;

            out.push(EmitCommand::Pen {
                x,
                y,
                pressure,
                touch: touching,
            });
        }

        out
    }
}
