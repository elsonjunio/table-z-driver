# TableZ Driver - Driver Rust para Mesa Digitalizadora

<div align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://github.com/elsonjunio/table-z-driver/blob/main/table_image.png?raw=true">
    <source media="(prefers-color-scheme: light)" srcset="https://github.com/elsonjunio/table-z-driver/blob/main/table_image.png?raw=true">
    <img alt="TableZ Driver"
         src="https://github.com/elsonjunio/table-z-driver/blob/main/table_image.png?raw=true"
         width="50%">
  </picture>

</div>

### 📖 Sobre o Projeto

Este é um projeto de estudos desenvolvido em Rust com o objetivo de aprender a linguagem e o framework **Tauri** para criação de aplicações desktop. O projeto permite usar mesas digitalizadoras de baixo custo (como a Zinnia MT100) no Linux, que normalmente não possuem suporte nativo.

O código é baseado no projeto original em Python [10moons-driver](https://github.com/alex-s-v/10moons-driver) de Alexandr Vasilyev, que permite utilizar essas mesas em ambiente Linux.

### 🎯 Objetivos de Aprendizado

- Domínio da linguagem Rust e seus conceitos de ownership/borrowing
- Desenvolvimento de aplicações desktop com Tauri
- Criação de interfaces gráficas com React + Tailwind CSS

### 🛠️ Tecnologias Utilizadas

- Backend: Rust, Tauri
- Frontend: React, TypeScript, Tailwind CSS
- Protocolo: USB HID
- Sistema: Linux (Ubuntu/Debian)

### 📋 Pré-requisitos

**Dependências do Linux**

```bash

sudo apt update
sudo apt install libusb-1.0-0-dev pkg-config build-essential
```

**Node.js e npm**

Recomenda-se o uso do Node.js v20 ou superior através do nvm:

```bash

# Instalar nvm (se não tiver)
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash

# Instalar e usar Node.js v20
nvm install 20
nvm use 20
```

### 🚀 Compilação e Execução

Estrutura do Projeto

```text

.
├── Cargo.toml          # Configuração do Rust
├── driver/             # Código do driver em Rust
├── table_z_config/     # Biblioteca de configuração
├── ui/                 # Interface gráfica (Tauri + React)
├── table_z_utils.yaml  # Arquivo de configuração
└── target/             # Build do Rust
```

**Compilação**

1 - Clone o repositório:

```bash

git clone https://github.com/elsonjunio/mt100_driver
cd tablet_driver_rust
```

2 - Compilar o projeto:

```bash

# Desenvolvimento (com hot-reload)
cd ui
npm install
npm run tauri dev

# Ou usando cargo watch para desenvolvimento do backend
cargo watch -x run
```

3 - Build de produção:

```bash

cd ui
npm run tauri build
```

**Execução em Desenvolvimento**


Para desenvolvimento simultâneo do frontend e backend:

Terminal 1 - Backend Rust:
```bash

cargo watch -x run
```

Terminal 2 - Frontend Tauri:
```bash

cd ui
npm run tauri dev
```

### ⚙️ Configuração

O arquivo de configuração principal é `table_z_utils.yaml`, que devem estar em:

- $HOME/table_z_utils.yaml (customizações do usuário)
- /etc/table_z_utils.yaml (global)

**Exemplo de Configuração**

```yaml

xinput_name: "TableZ Tablet"
vendor_id: 0x0b57
product_id: 0x1021
interface: 0

pen:
  max_x: 4096
  max_y: 4096
  max_pressure: 2048
  resolution_x: 100
  resolution_y: 100

actions:
  pen: "KEY_LEFTMOUSE"
  stylus: "KEY_RIGHTMOUSE"
  pen_touch: "None"
  tablet_buttons:
    - "KEY_LEFTCTRL+KEY_Z"
    - "KEY_LEFTCTRL+KEY_Y"

settings:
  swap_axis: false
  swap_direction_x: false
  swap_direction_y: false
```

### 🎮 Funcionalidades
**✅ Implementadas**

- Driver para mesas digitalizadoras compatíveis
- Interface gráfica com Tauri + React
- Configuração de eixos (inversão X/Y)
- Mapeamento de botões físicos
- Monitoramento em tempo real do status
- Visualização da posição e pressão da caneta
- System tray integration


**🎯 Oportunidades de Evolução**
1. Melhor Customização de Botões
    - Customização nos botões da caneta

2. Sistema de Macros
    - Gravação de macros
    - Mapear área da mesa como botões

3. Mecânicas Avançadas
    - Sensibilidade por pressão dinâmica
    - Modos de trabalho (arte, escrita, navegação)


### 🔧 Troubleshooting
Problemas Comuns

Permissão USB:
```bash

# Adicionar usuário ao grupo plugdev
sudo usermod -a -G plugdev $USER

# Ou criar regra udev personalizada
echo 'SUBSYSTEM=="usb", ATTR{idVendor}=="0b57", ATTR{idProduct}=="1021", MODE="0666"' | sudo tee /etc/udev/rules.d/99-tablet-zinnia.rules
sudo udevadm control --reload-rules
```

**Dispositivo não detectado:**
    Verifique se a mesa está conectada via USB
    Confirme vendor_id e product_id com lsusb
    Reinicie o serviço udev: sudo service udev restart


### 🙏 Agradecimentos

- Alexandr Vasilyev pelo projeto original 10moons-driver
