use bitflags::bitflags;
use port::Port;

pub const PIC_1_OFFSET: u8 = 32;

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

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u16)]
enum PicType {
    /// Master (IRQ 0-7)
    Pic1 = 0x20,

    /// Salve (IRQ 8 -15)
    Pic2 = 0xA0,
}

impl From<u16> for PicType {
    fn from(num: u16) -> Self {
        match num {
            0x20 => Self::Pic1,
            0xA0 => Self::Pic2,
            _ => panic!("Not a PIC"),
        }
    }
}

pub struct Pics {
    pub main: Pic,
    pub secondary: Pic,
}

impl Pics {
    pub fn end_of_interrupt(&mut self, index: usize) {
        // we always send it to the master but the slave too if it came from there
        if index >= 8 {
            self.secondary.end_of_interrupt();
        }

        self.main.end_of_interrupt();
    }

    pub fn remap(&mut self) {
        self.main.remap();
        self.secondary.remap();
    }

    pub fn disable(&mut self) {
        self.main.disable();
        self.secondary.disable();
    }

    pub fn mask_all(&mut self) {
        self.main.set_mask_all();
        self.secondary.set_mask_all();
    }
}

impl Default for Pics {
    fn default() -> Self {
        Self {
            main: Pic::pic_1(),
            secondary: Pic::pic_2(),
        }
    }
}

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
        Self::new(PicType::Pic1 as u16)
    }

    /// Salve (IRQ 8 -15)
    pub fn pic_2() -> Self {
        Self::new(PicType::Pic2 as u16)
    }

    /// Disable the pic
    pub fn disable(&mut self) {
        unsafe {
            self.command.write(0xFF);
        }
    }

    fn pic_type(&self) -> PicType {
        self.command.port().into()
    }

    fn get_offset(&self) -> u8 {
        match self.pic_type() {
            PicType::Pic1 => PIC_1_OFFSET,
            PicType::Pic2 => PIC_1_OFFSET + 8,
        }
    }

    /// Remap the pic
    pub fn remap(&mut self) {
        unsafe {
            let mut wait_port: Port<u8> = Port::new(0x80);
            let mut wait = || wait_port.write(0);

            // save masks
            let mask: u8 = self.data.read();

            self.command.write(ICW1::INIT.bits() | ICW1::ICW4.bits());
            wait();

            // Vector offset
            self.data.write(self.get_offset());
            wait();

            match self.pic_type() {
                PicType::Pic1 => {
                    // Tell Master PIC that there is a slave PIC at IRQ2 (0000 0100)
                    self.data.write(4)
                }
                PicType::Pic2 => {
                    // tell Slave PIC its cascade identity (0000 0010)
                    self.data.write(2)
                }
            }
            wait();

            self.data.write(ICW4::M8086.bits());
            wait();

            // Restore mask
            self.data.write(mask);
        }
    }

    pub fn set_mask_all(&mut self) {
        unsafe {
            self.data.write(0xFF);
        }
    }

    pub fn clear_mask_all(&mut self) {
        unsafe {
            self.data.write(0xFF);
        }
    }

    pub fn end_of_interrupt(&mut self) {
        unsafe { self.command.write(0x20) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn pic_versions() {
        let pic1 = Pic::pic_1();
        let pic2 = Pic::pic_2();

        assert_eq!(pic1.pic_type(), PicType::Pic1);
        assert_eq!(pic2.pic_type(), PicType::Pic2);
    }
}
