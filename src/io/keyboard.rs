use super::port::Port;
pub struct Keyboard {
    port: Port<u8>,
}

impl Keyboard {
    pub fn new() -> Self {
        Self {
            port: Port::new(0x60),
        }
    }

    fn read(&self) -> u8 {
        unsafe { self.port.read() }
    }

    pub fn get_key(&self) -> Option<char> {
        let scan_code = self.read();
        let key = match scan_code {
            0x02 => '1',
            0x03 => '2',
            0x04 => '3',
            0x05 => '4',
            0x06 => '5',
            0x07 => '6',
            0x08 => '7',
            0x09 => '8',
            0x0a => '9',
            0x0b => '0',
            _ => return None,
        };
        Some(key)
    }
}

impl Default for Keyboard {
    fn default() -> Self {
        Self::new()
    }
}
