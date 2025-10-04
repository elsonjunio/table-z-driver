use rusb::{Context, Device, Error as UsbError};
use anyhow::{bail, Result};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;

/// Estrutura responsável por realizar leituras contínuas de um endpoint USB.
///
/// A leitura ocorre em uma *thread* separada e envia os dados recebidos
/// para o *callback* fornecido pelo usuário.
///
/// Essa implementação é voltada principalmente para dispositivos HID
/// ou dispositivos que usam *interrupt endpoints* para envio periódico de dados.
pub struct USBReader;

impl USBReader {
    /// Cria uma nova instância do leitor USB.
    ///
    /// # Exemplo
    /// ```
    /// let reader = USBReader::new()?;
    /// ```
    pub fn new() -> Result<Self> {
        Ok(USBReader)
    }

    /// Inicia a leitura contínua de um endpoint USB.
    ///
    /// Este método:
    /// - Localiza o endpoint informado.
    /// - Detacha o *kernel driver* (se necessário).
    /// - Faz o *claim* da interface correspondente.
    /// - Inicia uma *thread* que realiza leituras periódicas até que `stop_flag` seja `false`.
    ///
    /// # Parâmetros
    /// - `device`: Dispositivo USB já detectado via `rusb`.
    /// - `endpoint`: Endereço do endpoint a ser lido (ex: `0x81`).
    /// - `stop_flag`: Flag atômica que controla o loop de leitura (quando `false`, encerra).
    /// - `callback`: Função que será chamada sempre que um pacote for recebido.
    ///
    /// # Retorno
    /// Retorna `Ok(())` se a thread de leitura foi iniciada com sucesso.
    ///
    /// # Erros
    /// Retorna erro (`anyhow::Error`) se o endpoint não for encontrado
    /// ou se ocorrer falha na abertura ou *claim* da interface.
    pub fn start<F>(
        &self,
        device: Device<Context>,
        endpoint: u8,
        stop_flag: Arc<AtomicBool>,
        mut callback: F,
    ) -> Result<()>
    where
        F: FnMut(Vec<u8>) + Send + 'static,
    {
        eprintln!("Iniciando leitura USB do endpoint {:#04x}", endpoint);

        let timeout = Duration::from_millis(500);
        let mut max_packet_size = 0usize;
        let mut iface_to_claim = None;

        // Obtém descritores de configuração e busca o endpoint alvo.
        let config_desc = device.active_config_descriptor()?;
        'outer: for (i, interface) in config_desc.interfaces().enumerate() {
            for (j, descriptor) in interface.descriptors().enumerate() {
                println!("Interface {} Descriptor {}", i, j);
                for ep in descriptor.endpoint_descriptors() {
                    if endpoint == ep.address() {
                        max_packet_size = ep.max_packet_size() as usize;
                        iface_to_claim = Some(descriptor.interface_number());
                        break 'outer;
                    }
                }
            }
        }

        if max_packet_size == 0 || iface_to_claim.is_none() {
            bail!("Endpoint {:#04x} não encontrado", endpoint);
        }

        let iface = iface_to_claim.unwrap();
        let handle = device.open()?;

        // Libera o driver do kernel, se ativo.
        if handle.kernel_driver_active(iface)? {
            handle.detach_kernel_driver(iface).ok();
        }

        handle.set_active_configuration(1).ok();
        handle.claim_interface(iface)?;

        // Cria thread de leitura
        thread::spawn(move || {
            println!("🟢 Thread de leitura iniciada (endpoint {:#04x})", endpoint);
            while stop_flag.load(Ordering::SeqCst) {
                let mut buf = vec![0u8; max_packet_size];

                match handle.read_interrupt(endpoint, &mut buf, timeout) {
                    Ok(size) => {
                        buf.truncate(size);
                        callback(buf);
                    }
                    Err(UsbError::Timeout) => {
                        // Tempo esgotado — loop continua
                    }
                    Err(e) => {
                        eprintln!("⚠️ Erro na leitura USB: {:?}", e);
                        break;
                    }
                }
            }
            println!("🔴 Thread de leitura encerrada (endpoint {:#04x})", endpoint);
        });

        Ok(())
    }
}
