use core::fmt;
use port::Port;

static COM1: u16 = 0x3F8;
type Register = Port<u8>;

pub struct Uart {
    /// Data register
    /// or (DLAB == 1)
    /// The least significant byte of the divisor value for setting the baud rate
    data: Register,
    /// Interrupts Enable Register
    /// or (DLAB == 1)
    /// the most significant byte of the divisor value
    int_en: Register,
    /// Interrupt Identification and FIFO control registers
    fifo_ctrl: Register,
    /// Line Control Register. The most significant bit of this register is the DLAB.
    line_ctrl: Register,
    /// Modem Control Register.
    modem_ctrl: Register,
    /// Line Status Register.
    line_sts: Register,
    /// Modem Status Register.
    modem_sts: Register,
    /// Scratch Register
    scratch: Register,

    // the current column
    column: usize,
}

impl Uart {
    pub fn new(com: u16) -> Self {
        Self {
            data: Port::new(com),
            int_en: Port::new(com + 1),
            fifo_ctrl: Port::new(com + 2),
            line_ctrl: Port::new(com + 3),
            modem_ctrl: Port::new(com + 4),
            line_sts: Port::new(com + 5),
            modem_sts: Port::new(com + 6),
            scratch: Port::new(com + 7),
            column: 0,
        }
    }

    /// # Safety
    /// The correct things to write according to osdev wiki
    pub unsafe fn init(&mut self) {
        self.int_en.write(0x00); // Disable all interrupts
        self.line_ctrl.write(0x80); // Enable DLAB (set baud rate divisor)
        self.data.write(0x03); // Set divisor to 3 (lo byte) 38400 baud
        self.int_en.write(0x00); //                  (hi byte)
        self.line_ctrl.write(0x03); // 8 bits, no parity, one stop bit
        self.fifo_ctrl.write(0xC7); // Enable FIFO, clear them, with 14-byte threshold
        self.modem_ctrl.write(0x0B); // IRQs enabled, RTS/DSR set
        self.modem_ctrl.write(0x1E); // Set in loopback mode, test the serial chip
        self.data.write(0xAE); // Test serial chip (send byte 0xAE and check if serial returns same byte)

        // Check if serial is faulty (i.e: not same byte as sent)
        if self.data.read() != 0xAE {
            panic!("Serial communication is faulty");
        }

        // If serial is not faulty set it in normal operation mode
        // (not-loopback with IRQs enabled and OUT#1 and OUT#2 bits enabled)
        self.modem_ctrl.write(0x0F);
    }

    pub fn disable(&mut self) {
        unsafe {
            self.modem_ctrl.write(0x1E); // Set in loopback mode
        }
    }

    pub fn enable(&mut self) {
        unsafe {
            self.modem_ctrl.write(0x0F); // Set in loopback mode
        }
    }

    fn write_byte(&mut self, byte: u8) {
        if self.column >= 80 {
            unsafe { self.data.write(b'\n') };
            self.column = 0;
        }

        if byte == b'\n' {
            self.column = 0;
        }

        if byte == b'\x08' {
            self.column = self.column.saturating_sub(2);
        }

        unsafe {
            while self.line_sts.read() & 0x20 == 0x0 {
                core::hint::spin_loop();
            }
            self.data.write(byte);
        }
        self.column += 1;
    }

    fn write_string(&mut self, string: &str) {
        for byte in string.bytes() {
            self.write_byte(byte);
            if byte == b'\x08' {
                self.write_byte(b' ');
                self.write_byte(b'\x08');
            }
        }
    }
}

impl Default for Uart {
    fn default() -> Self {
        Self::new(COM1)
    }
}

impl fmt::Write for Uart {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}
