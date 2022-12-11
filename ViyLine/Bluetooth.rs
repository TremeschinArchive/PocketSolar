// | (c) 2022 Tremeschin, MIT License | ViyLine Project | //
use crate::*;

// Bluetooth
impl ViyLineApp {

    // Write a value on the BlueTooth module
    pub fn bluetoothWrite(&self, data: u8) {
        block_on(self.hc06.as_ref().unwrap().write(&self.writeCharacteristic.as_ref().unwrap(), &[data], WriteType::WithoutResponse)).unwrap();
    }

    // Read 8 bits from the bluetooth
    pub fn bluetoothRead(&self) -> u8 {
        return block_on(self.hc06.as_ref().unwrap().read(&self.readCharacteristic.as_ref().unwrap())).unwrap()[0];
    }
}

impl ViyLineApp {

    // Auto connect on HC06 module
    pub async fn findBluetooth(&mut self) {
        info!("Attempting Bluetooth connection");
        let manager = Manager::new().await.unwrap();
        let adapter_list = manager.adapters().await.unwrap();

        // For all found Bluetooth adapters
        for adapter in adapter_list.iter() {
            let adapterName = format!("{}", adapter.adapter_info().await.unwrap());

            info!("Adapter: {}", adapterName);

            // Scan for peripherals
            adapter.start_scan(ScanFilter::default()).await.unwrap();

            // FIXME: Wasm unreachable
            let peripherals = adapter.peripherals().await.unwrap_or(Vec::new());
            info!(":: List of Peripherals:");
            async_std::task::sleep(std::time::Duration::from_millis(500)).await;

            for peripheral in peripherals.iter() {
                let properties = peripheral.properties().await.expect("Can't get properties").unwrap();
                // let local_name = properties.unwrap().local_name.unwrap_or(String::from("Unknown Name"));

                let local_name = properties.local_name.unwrap_or(String::from("Unknown Name"));
                let mac = properties.address;
                info!("- Peripheral [MAC: {mac}] [{local_name}]");

                // Only connect to HC-06 ViyLine™ Hardware
                if local_name != self.viylineHardwareBluetoothDeviceName {continue;}
                info!("Match!");

                // Connect if not paired
                if !peripheral.is_connected().await.unwrap() {
                    if let Err(err) = peripheral.connect().await {
                        info!(" - ERROR: {}", err);
                        continue;
                    }
                }

                // Show info on name
                info!(" - {}", local_name);

                // Discover services and characteristics
                peripheral.discover_services().await.unwrap();
                let characteristics = Some(peripheral.characteristics().clone());
                self.writeCharacteristic = Some(characteristics.as_ref().unwrap().iter().find(|c| c.uuid == uuid_from_u16(0xFFE2)).unwrap().clone());
                self.readCharacteristic  = Some(characteristics.as_ref().unwrap().iter().find(|c| c.uuid == uuid_from_u16(0xFFE1)).unwrap().clone());

                // Assign bluetooth module variables
                self.hc06 = Some(peripheral.clone());
                return;
            }
        }
    }
}
