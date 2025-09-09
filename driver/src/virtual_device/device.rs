use std::sync::{Arc, Mutex};
use evdev::{
    AbsInfo, AbsoluteAxisType, AttributeSet, EventType, InputEvent, Key, UinputAbsSetup,
    uinput::VirtualDeviceBuilder,
};

use anyhow::Result;

/// Estrutura para o dispositivo virtual da caneta
#[derive(Clone)]
pub struct VPen {
    pub device: Arc<Mutex<evdev::uinput::VirtualDevice>>,
}

impl VPen {
    pub fn new(
        x_max: i32,
        y_max: i32,
        pressure_max: i32,
        res_x: i32,
        res_y: i32,
        keys: &[Key],
        name: &str,
    ) -> Result<Self> {
        // Criar configurações dos eixos absolutos
        let abs_x = UinputAbsSetup::new(
            AbsoluteAxisType::ABS_X,
            AbsInfo::new(0, 0, x_max, 0, 0, res_x),
        );
        let abs_y = UinputAbsSetup::new(
            AbsoluteAxisType::ABS_Y,
            AbsInfo::new(0, 0, y_max, 0, 0, res_y),
        );
        let abs_pressure = UinputAbsSetup::new(
            AbsoluteAxisType::ABS_PRESSURE,
            AbsInfo::new(0, 0, pressure_max, 0, 0, 0),
        );

        // Criar device virtual
        let dev = VirtualDeviceBuilder::new()?
            .name(name)
            .with_keys(&AttributeSet::from_iter(keys.iter().cloned()))?
            .with_absolute_axis(&abs_x)?
            .with_absolute_axis(&abs_y)?
            .with_absolute_axis(&abs_pressure)?
            .build()?;

        Ok(Self {
            device: Arc::new(Mutex::new(dev)),
        })
    }

    /// Emitir eventos de posição/pressão
    pub fn emit(&self, x: i32, y: i32, pressure: i32, touch: bool) -> Result<(), std::io::Error> {
        let events = [
            InputEvent::new(EventType::ABSOLUTE, AbsoluteAxisType::ABS_X.0, x),
            InputEvent::new(EventType::ABSOLUTE, AbsoluteAxisType::ABS_Y.0, y),
            InputEvent::new(EventType::ABSOLUTE, AbsoluteAxisType::ABS_PRESSURE.0, pressure),
            InputEvent::new(
                EventType::KEY,
                Key::BTN_TOUCH.code(),
                if touch { 1 } else { 0 },
            ),
        ];

        let mut dev = self.device.lock().unwrap();
        dev.emit(&events)?;
        Ok(())
    }
}



/// Estrutura para os botões da mesa
pub struct VBtn {
    pub device: evdev::uinput::VirtualDevice,
}

impl VBtn {
    pub fn new(keys: &[Key], name: &str) -> Result<Self> {
        let mut dev = VirtualDeviceBuilder::new()?
            .name(name)
            .with_keys(&AttributeSet::from_iter(keys.iter().cloned()))?
            .build()?;

        Ok(Self { device: dev })
    }

    /// Emite evento de tecla
    pub fn emit(&mut self, key: Key, value: i32) -> Result<()> {
        let event = InputEvent::new(EventType::KEY, key.code(), value);
        self.device.emit(&[event])?;
        Ok(())
    }
}
