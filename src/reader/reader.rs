use rusb::{Context, Device, DeviceHandle, Hotplug, HotplugBuilder, UsbContext};
use anyhow::bail;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

pub struct USBReader ;

impl USBReader {
    pub fn new ()  -> anyhow::Result<Self> {
        Ok(USBReader)
    }
    pub fn start<F>(&self, device: Device<Context>, endpoint: u8, stop_flag: Arc<AtomicBool>, mut callback: F) -> anyhow::Result<()> 
    where
    F: FnMut(Vec<u8>) + Send + 'static,
    {
        eprintln!("{:?}", device);

        let timeout = std::time::Duration::from_millis(500);

        let mut max_packet_size: usize = 0;
        let mut iface_to_claim: Option<u8> = None;

        let config_desc = device.active_config_descriptor().unwrap();
        'outer: for (i, interface) in config_desc.interfaces().enumerate() {
            for (j, descriptor) in interface.descriptors().enumerate() {
                println!("Interface {} Descriptor {}", i, j);
                for ep in descriptor.endpoint_descriptors() {
                    if endpoint == ep.address() {
                        max_packet_size = ep.max_packet_size() as usize;
                        iface_to_claim = Some(descriptor.interface_number());
                        break 'outer; // sai de todos os loops
                    }
                }
            }
        }

        if max_packet_size == 0 || iface_to_claim.is_none() {
            bail!("Endpoint {:#04x} não encontrado", endpoint);
        }

        let iface = iface_to_claim.unwrap();
        let handle = device.open()?;

        // detach apenas da interface que vamos usar
        if handle.kernel_driver_active(iface)? {
            handle.detach_kernel_driver(iface).ok();
        }

        handle.set_active_configuration(1).ok();
        handle.claim_interface(iface)?;

        //loop {
        //    let mut buf = vec![0u8; max_packet_size];
        //    match handle.read_interrupt(endpoint, &mut buf, timeout) {
        //        Ok(size) => {
        //            buf.truncate(size);
        //            println!("Pacote: {:?}", buf);
        //            callback(buf);
        //        }
        //        Err(rusb::Error::Timeout) => {
        //            // nada chegou, normal
        //        }
        //        Err(e) => {
        //            eprintln!("Erro na leitura: {:?}", e);
        //             break; // sai do loop
        //        }
        //    }
        //}

        // Thread de leitura
        std::thread::spawn(move || {
            while stop_flag.load(Ordering::SeqCst) {
                let mut buf = vec![0u8; max_packet_size];
                match handle.read_interrupt(endpoint, &mut buf, timeout) {
                    Ok(size) => {
                        buf.truncate(size);
                        callback(buf);
                    }
                    Err(rusb::Error::Timeout) => {
                        // normal
                    }
                    Err(e) => {
                        eprintln!("Erro na leitura: {:?}", e);
                        break;
                    }
                }
            }
            println!("Thread de leitura terminada.");
        });

        Ok(())
    }

}
