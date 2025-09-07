//use rusb::Context;
use std::{thread, time};
use rusb::{Context, UsbContext};


fn main() {
    // Criar contexto USB
    let context = match Context::new() {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Erro ao criar contexto USB: {:?}", e);
            return;
        }
    };

    let vendor_id = 0x08f2;
    let product_id = 0x6811;

    let mut connected = false;

    loop {
        //let devices = context.devices().unwrap();

        // Listar dispositivos
        let devices = match context.devices() {
            Ok(devs) => devs,
            Err(e) => {
                eprintln!("Erro ao listar dispositivos USB: {:?}", e);
                return;
            }
        };


        let found = devices.iter().any(|d| {
            let desc = d.device_descriptor().unwrap();
            desc.vendor_id() == vendor_id && desc.product_id() == product_id
        });

        if found && !connected {
            println!("Device conectado!");
            connected = true;
        } else if !found && connected {
            println!("Device desconectado!");
            connected = false;
        }

        thread::sleep(time::Duration::from_secs(1));
    }
}
