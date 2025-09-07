use rusb::{Context, DeviceHandle, UsbContext};

pub struct USBReader {
    handle: DeviceHandle<Context>,
    endpoint: u8,
    max_packet_size: usize,
}

impl USBReader {
    pub fn new(vendor_id: u16, product_id: u16) -> anyhow::Result<Self> {
        let context = Context::new()?;
        let device = context
            .devices()?
            .iter()
            .find(|d| {
                let desc = d.device_descriptor().unwrap();
                desc.vendor_id() == vendor_id && desc.product_id() == product_id
            })
            .ok_or_else(|| anyhow::anyhow!("Device not found"))?;

        let desc = device.device_descriptor()?;
        let handle = device.open()?;

        // Interface 2, endpoint 0 (igual ao Python)
        let endpoint = 0x81; // normalmente IN endpoint = 0x81
        let max_packet_size = device
            .active_config_descriptor()?
            .interfaces()
            .nth(2)
            .unwrap()
            .descriptors()
            .next()
            .unwrap()
            .endpoint_descriptors()
            .next()
            .unwrap()
            .max_packet_size() as usize;

        Ok(USBReader {
            handle,
            endpoint,
            max_packet_size,
        })
    }

    pub fn read_packet(&mut self) -> anyhow::Result<Vec<u8>> {
        let mut buf = vec![0u8; self.max_packet_size];
        let timeout = std::time::Duration::from_millis(500);
        let size = self.handle.read_interrupt(self.endpoint, &mut buf, timeout)?;
        buf.truncate(size);
        Ok(buf)
    }
}
