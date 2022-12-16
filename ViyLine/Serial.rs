// | (c) 2022 Tremeschin, MIT License | ViyLine Project | //
use crate::*;

// Serial port
impl ViyLineApp {

    // Open the COM serial port
    pub fn openSerialPort(&mut self, portName: &String) {
        match serialport::new(portName, BAUDRATE).open() {
            Ok(port) => self.serialPort = Some(port),
            _ => error!("Failed to open SerialPort [{portName}]"),
        }
    }

    // Read 8 bits from serial port
    pub fn serialPortRead(&mut self) -> u8 {
        let mut serialBuffer: Vec<u8> = vec![0; 1];
        self.serialPort.as_mut().unwrap().read(serialBuffer.as_mut_slice());
        return serialBuffer[0];
    }

    // Write 8 bits to serial port
    pub fn serialPortWrite(&mut self, data: u8) {
        self.serialPort.as_mut().unwrap().write(&[data]);
    }
}
