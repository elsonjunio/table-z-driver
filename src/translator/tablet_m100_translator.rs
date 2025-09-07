use crate::translator::translator::{EmitCommand, Translator};

pub struct TabletM100Translator;

impl Translator for TabletM100Translator {
     fn conv(&self, buf: &Vec<u8>) -> Vec<EmitCommand> {
        let mut out = Vec::new();

        if buf.len() >= 8 && (buf[1] == 192 || buf[1] == 193) {
            let pen_x = (buf[5] as i32 * 255 + buf[4] as i32);
            let pen_y = (buf[3] as i32 * 255 + buf[2] as i32);
            let pen_pressure = (buf[7] as i32 * 255 + buf[6] as i32);
            let touching = buf[1] != 192;

            out.push(EmitCommand::Pen {
                x: pen_x,
                y: pen_y,
                pressure: pen_pressure,
                touch: touching,
            });
        }

        out
    }
}