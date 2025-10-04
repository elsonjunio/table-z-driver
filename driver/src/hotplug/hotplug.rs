use rusb::{Context, Device, Hotplug, HotplugBuilder, UsbContext};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// Evento personalizado de hotplug USB.
///
/// Representa os dois principais eventos que o sistema pode detectar:
/// - `DeviceArrived`: um dispositivo USB foi conectado.
/// - `DeviceLeft`: um dispositivo USB foi removido.
#[derive(Debug, Clone, Copy)]
pub enum CustomHotplugEvent {
    DeviceArrived,
    DeviceLeft,
}

/// Estrutura que trata eventos de hotplug.
///
/// Internamente, mantém um callback compartilhado e protegido por `Mutex`
/// para permitir chamadas concorrentes seguras entre threads.
pub struct HotPlugHandler {
    /// Função callback chamada em cada evento USB.
    ///
    /// O callback recebe o dispositivo e o tipo de evento (`arrived` ou `left`).
    callback: Arc<Mutex<dyn FnMut(Device<Context>, CustomHotplugEvent) + Send>>,
}

impl Hotplug<Context> for HotPlugHandler {
    /// Chamado automaticamente quando um novo dispositivo USB é conectado.
    fn device_arrived(&mut self, device: Device<Context>) {
        (self.callback.lock().unwrap())(device, CustomHotplugEvent::DeviceArrived);
    }

    /// Chamado automaticamente quando um dispositivo USB é desconectado.
    fn device_left(&mut self, device: Device<Context>) {
        (self.callback.lock().unwrap())(device, CustomHotplugEvent::DeviceLeft);
    }
}

impl HotPlugHandler {
    /// Inicializa o monitoramento de eventos USB (hotplug).
    ///
    /// Esta função inicia uma *thread* dedicada para observar eventos de conexão
    /// e desconexão de dispositivos USB, chamando o `callback` fornecido sempre
    /// que um evento é detectado.
    ///
    /// # Exemplo
    /// ```
    /// HotPlugHandler::init(|device, event| {
    ///     match event {
    ///         CustomHotplugEvent::DeviceArrived => println!("Novo dispositivo: {:?}", device),
    ///         CustomHotplugEvent::DeviceLeft => println!("Dispositivo removido: {:?}", device),
    ///     }
    /// });
    /// ```
    pub fn init<F>(callback: F)
    where
        F: FnMut(Device<Context>, CustomHotplugEvent) + Send + 'static,
    {
        // Verifica se o suporte a hotplug está disponível na versão da libusb.
        if !rusb::has_hotplug() {
            eprintln!("⚠️ libusb hotplug não suportado nesta versão.");
            return;
        }

        let cb = Arc::new(Mutex::new(callback));

        thread::spawn(move || {
            let context = match Context::new() {
                Ok(ctx) => ctx,
                Err(e) => {
                    eprintln!("❌ Erro ao criar contexto USB: {:?}", e);
                    return;
                }
            };

            // Registra o handler de hotplug
            let _registration = match HotplugBuilder::new()
                .enumerate(true)
                .register(
                    &context,
                    Box::new(HotPlugHandler {
                        callback: cb.clone(),
                    }),
                ) {
                Ok(reg) => reg,
                Err(e) => {
                    eprintln!("❌ Falha ao registrar callback hotplug: {:?}", e);
                    return;
                }
            };

            println!("🔌 [Hotplug] Monitorando eventos USB...");

            // Loop de eventos principal
            loop {
                if let Err(e) = context.handle_events(Some(Duration::from_millis(200))) {
                    eprintln!("⚠️ Erro no handle_events: {:?}", e);
                    thread::sleep(Duration::from_secs(1)); // pequena pausa antes de tentar novamente
                }
            }
        });
    }
}
