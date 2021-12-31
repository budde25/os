use port::{Port, PortWriteOnly};
pub struct Pit {
    channel_0: Port<u8>,
    channel_1: Port<u8>,
    channel_2: Port<u8>,
    mode_command: PortWriteOnly<u8>,
}

impl Pit {
    pub fn new() -> Self {
        const PORT_NUM: u16 = 0x40;
        Self {
            channel_0: Port::new(PORT_NUM),
            channel_1: Port::new(PORT_NUM + 1),
            channel_2: Port::new(PORT_NUM + 2),
            mode_command: PortWriteOnly::new(PORT_NUM + 3),
        }
    }

    pub fn sleep(&mut self) {
        unsafe { self.mode_command.write(0x30) };
        unsafe { self.channel_0.write(0xA9) };
        unsafe { self.channel_0.write(0x4) };
    }

    pub fn is_done(&mut self) -> bool {
        unsafe { self.mode_command.write(0xE2) };
        unsafe { self.channel_0.read() & 0b10000000 != 0 }
    }
}
