use evdev::{
    AbsInfo, AbsoluteAxisType, AttributeSet, EventType, InputEvent, Key, UinputAbsSetup,
    uinput::VirtualDeviceBuilder,
};
use std::sync::{Arc, Mutex};
use anyhow::Result;

/// Representa uma caneta virtual (pen) criada via `uinput`.
///
/// Este dispositivo é responsável por emitir eventos de posição (X/Y),
/// pressão e toque (`BTN_TOUCH`), simulando uma mesa digitalizadora.
///
/// ### Características
/// - Suporte a eixos absolutos (`ABS_X`, `ABS_Y`, `ABS_PRESSURE`)
/// - Pode ser usado em conjunto com um tradutor de pacotes USB
/// - É thread-safe via `Arc<Mutex<_>>`, permitindo uso concorrente
#[derive(Clone)]
pub struct VPen {
    /// Dispositivo virtual do `uinput`, protegido por `Mutex` para acesso seguro.
    pub device: Arc<Mutex<evdev::uinput::VirtualDevice>>,
}

impl VPen {
    /// Cria um novo dispositivo de caneta virtual configurado com os eixos e teclas fornecidos.
    ///
    /// # Parâmetros
    /// - `x_max`: Valor máximo do eixo X
    /// - `y_max`: Valor máximo do eixo Y
    /// - `pressure_max`: Valor máximo da pressão detectável
    /// - `res_x`: Resolução (DPI) no eixo X
    /// - `res_y`: Resolução (DPI) no eixo Y
    /// - `keys`: Lista de teclas (geralmente contém `BTN_TOUCH`, `BTN_TOOL_PEN`, etc.)
    /// - `name`: Nome do dispositivo a ser criado (aparece em `/dev/input/by-id`)
    ///
    /// # Retorno
    /// Retorna `Result<Self>` com a instância de `VPen` pronta para uso.
    pub fn new(
        x_max: i32,
        y_max: i32,
        pressure_max: i32,
        res_x: i32,
        res_y: i32,
        keys: &[Key],
        name: &str,
    ) -> Result<Self> {
        // Configuração dos eixos absolutos (posição e pressão)
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

        // Criação do dispositivo virtual
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

    /// Emite um conjunto de eventos simulando movimento e toque da caneta.
    ///
    /// # Parâmetros
    /// - `x`: Posição X absoluta
    /// - `y`: Posição Y absoluta
    /// - `pressure`: Nível de pressão (0–`pressure_max`)
    /// - `touch`: `true` se houver contato (toque ativo)
    ///
    /// # Exemplo
    /// ```
    /// pen.emit(1200, 800, 300, true)?;
    /// ```
    pub fn emit(&self, x: i32, y: i32, pressure: i32, touch: bool) -> Result<(), std::io::Error> {
        let events = [
            InputEvent::new(EventType::ABSOLUTE, AbsoluteAxisType::ABS_X.0, x),
            InputEvent::new(EventType::ABSOLUTE, AbsoluteAxisType::ABS_Y.0, y),
            InputEvent::new(
                EventType::ABSOLUTE,
                AbsoluteAxisType::ABS_PRESSURE.0,
                pressure,
            ),
            InputEvent::new(
                EventType::KEY,
                Key::BTN_TOUCH.code(),
                if touch { 1 } else { 0 },
            ),
        ];

        // Bloqueia o dispositivo antes de enviar eventos
        let mut dev = self.device.lock().unwrap();
        dev.emit(&events)?;
        Ok(())
    }
}

/// Representa um dispositivo virtual de botões (sem eixos),
/// responsável por emitir eventos de teclas.
///
/// Usado para mapear os botões físicos do tablet para combinações de teclas.
#[derive(Clone)]
pub struct VBtn {
    /// Dispositivo virtual protegido por Mutex.
    pub device: Arc<Mutex<evdev::uinput::VirtualDevice>>,
}

impl VBtn {
    /// Cria um novo dispositivo virtual apenas com suporte a teclas.
    ///
    /// # Parâmetros
    /// - `keys`: Lista de teclas suportadas
    /// - `name`: Nome do dispositivo
    pub fn new(keys: &[Key], name: &str) -> Result<Self> {
        let dev = VirtualDeviceBuilder::new()?
            .name(name)
            .with_keys(&AttributeSet::from_iter(keys.iter().cloned()))?
            .build()?;

        Ok(Self {
            device: Arc::new(Mutex::new(dev)),
        })
    }

    /// Emite um evento de tecla pressionada ou liberada.
    ///
    /// # Parâmetros
    /// - `key`: Tecla a emitir
    /// - `value`: `true` para pressionar, `false` para soltar
    ///
    /// # Exemplo
    /// ```
    /// btn.emit(Key::KEY_A, true)?;  // Pressiona 'A'
    /// btn.emit(Key::KEY_A, false)?; // Solta 'A'
    /// ```
    pub fn emit(&self, key: Key, value: bool) -> Result<()> {
        let pressed = if value { 1 } else { 0 };
        let event = InputEvent::new(EventType::KEY, key.code(), pressed);
        let mut dev = self.device.lock().unwrap();
        dev.emit(&[event])?;
        Ok(())
    }
}
