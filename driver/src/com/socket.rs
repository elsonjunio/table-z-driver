use std::{
    os::unix::net::{UnixListener, UnixStream},
    io::{Read, Write},
    sync::{Arc, Mutex, mpsc::{self, Sender, Receiver}},
    thread,
    fs,
    path::Path,
};

pub struct SocketServer {
    tx_broadcast: Sender<Vec<u8>>,
    rx_commands: Receiver<String>,
}

impl SocketServer {
    pub fn new(path: &str) -> Arc<Self> {
        // remove o socket antigo, se existir
        if Path::new(path).exists() {
            fs::remove_file(path).unwrap_or(());
        }

        let (tx_broadcast, rx_broadcast) = mpsc::channel::<Vec<u8>>();
        let (tx_commands, rx_commands) = mpsc::channel::<String>();

        let server = Arc::new(Self {
            tx_broadcast,
            rx_commands,
        });

        let server_clone = Arc::clone(&server);

        //
        let path = path.to_string();

        thread::spawn(move || {
            let listener = UnixListener::bind(&path).expect("Não conseguiu abrir Unix socket");
            println!("Unix socket server escutando em {}", &path);

            let clients: Arc<Mutex<Vec<UnixStream>>> = Arc::new(Mutex::new(Vec::new()));

            // Thread para aceitar conexões
            {
                let clients = Arc::clone(&clients);
                let tx_commands = tx_commands.clone();

                thread::spawn(move || {
                    for stream in listener.incoming() {
                        match stream {
                            Ok(stream) => {
                                println!("Novo cliente conectado ao socket");

                                stream.set_nonblocking(true).unwrap();
                                clients.lock().unwrap().push(stream.try_clone().unwrap());

                                // Thread para leitura de mensagens do cliente
                                let mut client = stream.try_clone().unwrap();
                                let tx = tx_commands.clone();
                                thread::spawn(move || {
                                    let mut buf = [0u8; 1024];
                                    loop {
                                        match client.read(&mut buf) {
                                            Ok(0) => break, // cliente fechou
                                            Ok(n) => {
                                                let msg = String::from_utf8_lossy(&buf[..n]).to_string();
                                                println!("Comando recebido: {}", msg);
                                                tx.send(msg).unwrap();
                                            }
                                            Err(_) => {
                                                thread::sleep(std::time::Duration::from_millis(200));
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

            // Thread para broadcast
            loop {
                if let Ok(packet) = rx_broadcast.recv() {
                    let mut guard = clients.lock().unwrap();
                    guard.retain_mut(|client| {
                        if let Err(e) = client.write_all(&packet) {
                            eprintln!("Erro enviando para cliente: {:?}", e);
                            return false;
                        }
                        true
                    });
                }
            }
        });

        server_clone
    }

    pub fn sender(&self) -> Sender<Vec<u8>> {
        self.tx_broadcast.clone()
    }

    pub fn try_recv_command(&self) -> Option<String> {
        self.rx_commands.try_recv().ok()
    }
}
