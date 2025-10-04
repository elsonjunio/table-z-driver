use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use table_z_config::Config;

/// Representa um comando interpretado e pronto para ser emitido pelo sistema.
///
/// Esses comandos são normalmente produzidos por um [`Translator`],
/// que converte pacotes binários recebidos via USB em ações semânticas.
///
/// Cada variante representa uma ação lógica detectada no dispositivo —
/// como movimento da caneta ou pressionamento de botão.
#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub enum EmitCommand {
    /// Evento da caneta, representando posição e pressão atuais.
    Pen {
        /// Coordenada X do toque (normalmente em pixels ou unidades absolutas).
        x: i32,
        /// Coordenada Y do toque.
        y: i32,
        /// Valor da pressão da caneta.
        pressure: i32,
        /// Indica se há contato (toque) ativo.
        touch: bool,
    },

    /// Evento de botão físico no tablet.
    Btn {
        /// Código da tecla (keycode, normalmente compatível com X11 ou HID).
        key: i32,
        /// Indica se o botão foi pressionado (`true`) ou solto (`false`).
        pressed: bool,
        /// Índice do botão físico na mesa digitalizadora.
        index: usize,
    },
}

/// Trait responsável por traduzir pacotes USB crus em comandos de alto nível.
///
/// Essa trait permite implementar tradutores específicos para diferentes modelos de tablet
/// ou dispositivos, mantendo uma interface genérica e extensível.
///
/// O implementador dessa trait normalmente analisa bytes do firmware do dispositivo
/// e retorna uma lista de [`EmitCommand`] representando os eventos reconhecidos.
pub trait Translator: Send + Sync {
    /// Converte um pacote binário (raw USB data) em uma lista de [`EmitCommand`].
    ///
    /// # Parâmetros
    /// - `buf`: Buffer recebido diretamente da USB (ex: leitura via `rusb::read_interrupt`).
    ///
    /// # Retorno
    /// Uma lista de comandos interpretados, prontos para serem processados ou emitidos.
    ///
    /// # Exemplo
    /// ```
    /// let commands = translator.conv(&packet);
    /// for cmd in commands {
    ///     println!("Evento: {:?}", cmd);
    /// }
    /// ```
    fn conv(&self, buf: &Vec<u8>) -> Vec<EmitCommand>;

    /// Atualiza a lógica de tradução a partir de uma nova configuração.
    ///
    /// # Parâmetros
    /// - `cfg`: Estrutura [`Config`] contendo dados como mapeamento de botões e resolução da mesa.
    ///
    /// Essa função permite reconfigurar o tradutor sem precisar reinicializá-lo,
    /// por exemplo, quando o usuário altera preferências no software.
    fn update_from_config(&mut self, cfg: &Config);
}
