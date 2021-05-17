use core::marker::PhantomData;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

/// A port
#[derive(Debug, Clone, Copy)]
pub struct Port<T> {
    port: u16,
    // phantom data allows type T
    phantom: PhantomData<T>,
}

impl<T: PortIO> Port<T> {
    pub fn new(port: u16) -> Self {
        Self {
            port,
            phantom: PhantomData,
        }
    }

    pub unsafe fn write(&self, value: T) {
        T::write(self.port, value);
    }

    pub unsafe fn read(&self) -> T {
        T::read(self.port)
    }
}

/// A port IO trait that lets us define what types can be used with port I/O
pub trait PortIO {
    /// lets you write to the port
    unsafe fn write(port: u16, value: Self);
    /// let you read from the port
    unsafe fn read(port: u16) -> Self;
}

impl PortIO for u8 {
    unsafe fn read(port: u16) -> Self {
        let output: Self;
        asm!("in al, dx", out("al") output, in("dx") port, options(nomem, nostack, preserves_flags));
        output
    }

    unsafe fn write(port: u16, value: Self) {
        asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack, preserves_flags));
    }
}

impl PortIO for u16 {
    unsafe fn read(port: u16) -> Self {
        let output: Self;
        asm!("in ax, dx", out("ax") output, in("dx") port, options(nomem, nostack, preserves_flags));
        output
    }

    unsafe fn write(port: u16, value: Self) {
        asm!("out dx, ax", in("dx") port, in("ax") value, options(nomem, nostack, preserves_flags));
    }
}

impl PortIO for u32 {
    unsafe fn read(port: u16) -> Self {
        let output: Self;
        asm!("in eax, dx", out("eax") output, in("dx") port, options(nomem, nostack, preserves_flags));
        output
    }

    unsafe fn write(port: u16, value: Self) {
        asm!("out dx, eax", in("dx") port, in("eax") value, options(nomem, nostack, preserves_flags));
    }
}