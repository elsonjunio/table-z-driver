use rusb::{Context, DeviceHandle, UsbContext};
//use rusb::{Context, UsbContext};


//pub struct USBReader {
//    handle: DeviceHandle<Context>,
//    endpoint: u8,
//    max_packet_size: usize,
//}

fn main() {


    let vendor_id = 0x08f2;
    let product_id= 0x6811;


    // Criar contexto USB
    let context = match Context::new() {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Erro ao criar contexto USB: {:?}", e);
            return;
        }
    };

    // Listar dispositivos
    let devices = match context.devices() {
        Ok(devs) => devs,
        Err(e) => {
            eprintln!("Erro ao listar dispositivos USB: {:?}", e);
            return;
        }
    };

    println!("Total de dispositivos USB encontrados: {}", devices.len());


    for device in devices.iter() {
        let desc = match device.device_descriptor() {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Erro ao obter descriptor do dispositivo: {:?}", e);
                continue;
            }
        };

        println!(
            "Bus {:03} Device {:03}: ID {:04x}:{:04x}",
            device.bus_number(),
            device.address(),
            desc.vendor_id(),
            desc.product_id()
        );
    }



    let device = devices
        .iter()
        .find(|d| {
            let desc = d.device_descriptor().unwrap();
            desc.vendor_id() == vendor_id && desc.product_id() == product_id
        })
        .ok_or_else(|| anyhow::anyhow!("Device not found")).unwrap();

    eprintln!("{:?}", device);

    let config_desc = device.active_config_descriptor().unwrap();
    for (i, interface) in config_desc.interfaces().enumerate() {
        for (j, descriptor) in interface.descriptors().enumerate() {
            println!("Interface {} Descriptor {}", i, j);
            for ep in descriptor.endpoint_descriptors() {
                println!(
                    "  Endpoint: addr={:#04x}, max_packet_size={}",
                    ep.address(),
                    ep.max_packet_size()
                );
            }
        }
    }

    let handle = device.open().unwrap();

//    // Libera o driver padrão do kernel
    for iface in 0..3 {
        if handle.kernel_driver_active(iface).unwrap_or(false) {
            handle.detach_kernel_driver(iface).ok();
        }
    }

//    // Seleciona configuração ativa
    handle.set_active_configuration(1).unwrap();

//    // Faz claim da interface que você quer usar (ex: 2)
    handle.claim_interface(2).unwrap();

//    let max_packet_size = device
//        .active_config_descriptor()
//        .unwrap()
//        .interfaces()
//        .nth(2)
//        .unwrap()
//        .descriptors()
//        .next()
//        .unwrap()
//        .endpoint_descriptors()
//        .next()
//        .unwrap()
//        .max_packet_size() as usize;

    let max_packet_size= 8;

    let endpoint = 0x85; // IN endpoint da mesa
    let timeout = std::time::Duration::from_millis(500);

    loop {
        let mut buf = vec![0u8; max_packet_size];
        match handle.read_interrupt(endpoint, &mut buf, timeout) {
            Ok(size) => {
                buf.truncate(size);
                println!("Pacote: {:?}", buf);
            }
            Err(rusb::Error::Timeout) => {
                // nada chegou, normal
            }
            Err(e) => {
                eprintln!("Erro na leitura: {:?}", e);
                 break; // sai do loop
            }
        }
    }


}