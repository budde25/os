use super::port::Port;

/// Master (IRQ 0-7)
const PIC_1: u16 = 0x20;

/// Salve (IRQ 8 -15)
const PIC_2: u16 = 0xA0;

/// Programmable Interrupt Controller
pub struct Pic {
    command: Port<u8>,
    data: Port<u8>,
}

impl Pic {
    /// Create a new PIC
    fn new(port: u16) -> Self {
        Self {
            command: Port::new(port),
            data: Port::new(port),
        }
    }

    /// Master (IRQ 0-7)
    pub fn pic_1() -> Self {
        Self::new(PIC_1)
    }

    /// Salve (IRQ 8 -15)
    pub fn pic_2() -> Self {
        Self::new(PIC_2)
    }

    /// Disable the pic
    pub fn diable(&mut self) {
        unsafe {
            self.command.write(0xFF);
        }
    }
}
