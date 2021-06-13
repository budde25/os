use super::port::Port;
use bitflags::bitflags;

bitflags! {
    pub struct ICW1: u8 {
        /// ICW4 (not) needed
        const ICW4 = 0x01;
        /// Single (cascade) mode
        const SINGLE = 0x2;
        /// Call address interval 4 (8)
        const INTERVAL4 = 0x4;
        /// pLevel triggered (edge) mode
        const LEVEL = 0x8;
        /// Initialization - required!
        const INIT = 0x10;
    }
}

bitflags! {
    struct ICW4: u8 {
        /// 8086/88 (MCS-80/85) mode
        const M8086 = 0x01;
        /// Auto (normal) EOI
        const AUTO = 0x2;
        /// Buffered mode/slave
        const BUF_SLAVE = 0x4;
        /// Buffered mode/master
        const BUF_MASTER = 0x8;
        /// Special fully nested (not)
        const SFNM = 0x10;
    }
}

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
            data: Port::new(port + 1),
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
    pub fn disable(&mut self) {
        unsafe {
            self.command.write(0xFF);
        }
    }

    /// Remap the pic
    pub fn remap(&mut self, offset: u8, master: bool) {
        unsafe {
            // save mask
            let _mask: u8 = self.data.read();
            self.command.write(ICW1::INIT.bits() | ICW1::ICW4.bits());
            self.data.write(offset);
            match master {
                true => self.data.write(4),
                false => self.data.write(2),
            }
        }
    }
}
