use uinput::event::absolute::Position;
use uinput::event::absolute::Pressure;
use uinput::event::keyboard;
use uinput::event::absolute;
use uinput::event::Event;
use uinput::Device;

use std::fs::File;
use std::time::Duration;

pub struct VirtualPen {
    dev: Device,
}

pub struct VirtualButtons {
    dev: Device,
}

impl VirtualPen {
    pub fn new(name: &str, max_x: i32, max_y: i32, max_pressure: i32, res_x: i32, res_y: i32, pen_codes: &[i32]) -> std::io::Result<Self> {
        let mut builder = uinput::default()?
            .name(name)?
            .event(Event::Key(keyboard::Key::Unknown))?; // inicializa com um evento genérico

        // adiciona teclas do pen
        for code in pen_codes {
            builder = builder.event(Event::Key(keyboard::Key::from(*code as u16)))?;
        }

        // adiciona eixos absolutos
        builder = builder.event(Event::Absolute(absolute::Position::X))?;
        builder = builder.event(Event::Absolute(absolute::Position::Y))?;
        builder = builder.event(Event::Absolute(absolute::Pressure::Pressure))?;

        // cria dispositivo
        let dev = builder.create()?;

        Ok(Self { dev })
    }

    pub fn write_abs(&mut self, code: i32, value: i32) -> std::io::Result<()> {
        self.dev.write(Event::Absolute(absolute::Absolute::from(code as u16)), value)?;
        Ok(())
    }

    pub fn write_key(&mut self, code: i32, value: i32) -> std::io::Result<()> {
        self.dev.write(Event::Key(keyboard::Key::from(code as u16)), value)?;
        Ok(())
    }

    pub fn sync(&mut self) -> std::io::Result<()> {
        self.dev.synchronize()?;
        Ok(())
    }
}

impl VirtualButtons {
    pub fn new(name: &str, btn_codes: &[i32]) -> std::io::Result<Self> {
        let mut builder = uinput::default()?
            .name(name)?
            .event(Event::Key(keyboard::Key::Unknown))?;

        for code in btn_codes {
            builder = builder.event(Event::Key(keyboard::Key::from(*code as u16)))?;
        }

        let dev = builder.create()?;
        Ok(Self { dev })
    }

    pub fn write_key(&mut self, code: i32, value: i32) -> std::io::Result<()> {
        self.dev.write(Event::Key(keyboard::Key::from(code as u16)), value)?;
        Ok(())
    }

    pub fn sync(&mut self) -> std::io::Result<()> {
        self.dev.synchronize()?;
        Ok(())
    }
}
