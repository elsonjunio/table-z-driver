use std::{
    fs,
    io::{Read, Write},
    os::unix::net::{UnixListener, UnixStream},
    path::Path,
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

/// Servidor de comunicação baseado em Unix Socket.
///
/// Este servidor permite:
/// - **Broadcast** de mensagens (dados binários) para todos os clientes conectados.
/// - **Recebimento** de comandos (strings) enviados por clientes.
///
/// Ele é útil para comunicação local entre processos (por exemplo, entre a UI e o daemon de driver da mesa).
///
/// ### Fluxo de funcionamento
/// 1. Cria o socket Unix e aceita conexões.
/// 2. Cada cliente recebe sua própria thread de leitura.
/// 3. Uma thread de broadcast envia pacotes para todos os clientes conectados.
/// 4. Mensagens recebidas dos clientes são publicadas via `rx_commands`.
pub struct SocketServer {
    /// Canal para enviar mensagens a todos os clientes.
    tx_broadcast: Sender<Vec<u8>>,
    /// Canal para receber comandos de clientes.
    rx_commands: Receiver<String>,
}

impl SocketServer {
    /// Cria e inicia o servidor de socket Unix.
    ///
    /// - `path`: caminho do socket (ex: `/tmp/tablet.sock`)
    ///
    /// Retorna um `Arc<SocketServer>` que pode ser compartilhado entre threads.
    pub fn new(path: &str) -> Arc<Self> {
        // Remove o arquivo de socket anterior, se existir
        if Path::new(path).exists() {
            if let Err(e) = fs::remove_file(path) {
                eprintln!("Aviso: não foi possível remover socket antigo: {:?}", e);
            }
        }

        let (tx_broadcast, rx_broadcast) = mpsc::channel::<Vec<u8>>();
        let (tx_commands, rx_commands) = mpsc::channel::<String>();

        let server = Arc::new(Self {
            tx_broadcast,
            rx_commands,
        });

        let server_ref = Arc::clone(&server);
        let socket_path = path.to_string();

        // Thread principal do servidor
        thread::spawn(move || {
            let listener = match UnixListener::bind(&socket_path) {
                Ok(listener) => {
                    println!("Servidor Unix socket escutando em {}", &socket_path);
                    listener
                }
                Err(e) => {
                    eprintln!("Erro ao iniciar socket em {}: {:?}", socket_path, e);
                    return;
                }
            };

            let clients: Arc<Mutex<Vec<UnixStream>>> = Arc::new(Mutex::new(Vec::new()));

            // Thread de aceitação de clientes
            {
                let clients = Arc::clone(&clients);
                let tx_commands = tx_commands.clone();

                thread::spawn(move || {
                    for stream in listener.incoming() {
                        match stream {
                            Ok(stream) => {
                                println!("Novo cliente conectado");

                                if let Err(e) = stream.set_nonblocking(true) {
                                    eprintln!("Erro ao configurar stream como non-blocking: {:?}", e);
                                    continue;
                                }

                                clients.lock().unwrap().push(stream.try_clone().unwrap());

                                // Thread de leitura para cada cliente
                                let mut client = match stream.try_clone() {
                                    Ok(c) => c,
                                    Err(e) => {
                                        eprintln!("Erro ao clonar stream: {:?}", e);
                                        continue;
                                    }
                                };

                                let tx = tx_commands.clone();

                                thread::spawn(move || {
                                    let mut buf = [0u8; 1024];
                                    loop {
                                        match client.read(&mut buf) {
                                            Ok(0) => {
                                                // Cliente fechou a conexão
                                                break;
                                            }
                                            Ok(n) => {
                                                let msg = String::from_utf8_lossy(&buf[..n]).to_string();
                                                println!("Comando recebido: {}", msg);
                                                if tx.send(msg).is_err() {
                                                    break; // receptor foi fechado
                                                }
                                            }
                                            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                                                thread::sleep(Duration::from_millis(200));
                                            }
                                            Err(e) => {
                                                eprintln!("Erro de leitura no cliente: {:?}", e);
                                                break;
                                            }
                                        }
                                    }
                                });
                            }
                            Err(e) => {
                                eprintln!("Erro ao aceitar cliente: {:?}", e);
                            }
                        }
                    }
                });
            }

            // Thread de broadcast
            loop {
                match rx_broadcast.recv() {
                    Ok(packet) => {
                        let mut clients_guard = clients.lock().unwrap();

                        // Remove clientes com erro de escrita
                        clients_guard.retain_mut(|client| {
                            if let Err(e) = client.write_all(&packet) {
                                eprintln!("Erro enviando para cliente: {:?}", e);
                                false
                            } else {
                                true
                            }
                        });
                    }
                    Err(_) => {
                        eprintln!("Canal de broadcast encerrado, servidor terminando.");
                        break;
                    }
                }
            }
        });

        server_ref
    }

    /// Retorna um `Sender` para enviar mensagens a todos os clientes conectados.
    pub fn sender(&self) -> Sender<Vec<u8>> {
        self.tx_broadcast.clone()
    }

    /// Tenta receber um comando enviado por algum cliente.
    /// Retorna `None` se não houver mensagens disponíveis.
    pub fn try_recv_command(&self) -> Option<String> {
        self.rx_commands.try_recv().ok()
    }
}
