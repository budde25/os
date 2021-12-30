use core::arch::asm;
use core::fmt;
use core::marker::PhantomData;

// credit: https://github.com/rust-osdev/x86_64/blob/master/src/instructions/port.rs

//// A port IO trait that lets us define what types can be used with port I/O
pub trait PortIO: PortRead + PortWrite {}

/// A port write trait that lets you write data to a port
pub trait PortWrite {
    /// lets you write to the port
    /// # Safety
    /// This function is unsafe
    unsafe fn write(port: u16, value: Self);
    unsafe fn writes(port: u16, buf: &[Self])
    where
        Self: Sized;
}

/// A port write trait that lets you read data to a port
pub trait PortRead {
    /// let you read from the port
    /// # Safety
    /// This function is unsafe
    unsafe fn read(port: u16) -> Self;
    unsafe fn reads(port: u16, buf: &mut [Self])
    where
        Self: Sized;
}

mod hidden {
    pub trait Access {
        const DEBUG_NAME: &'static str;
    }
}

pub trait PortReadAccess: hidden::Access {}
pub trait PortWriteAccess: hidden::Access {}

#[derive(Debug)]
pub struct ReadOnlyAccess(());

impl hidden::Access for ReadOnlyAccess {
    const DEBUG_NAME: &'static str = "ReadOnly";
}

impl PortReadAccess for ReadOnlyAccess {}

#[derive(Debug)]
pub struct WriteOnlyAccess(());

impl hidden::Access for WriteOnlyAccess {
    const DEBUG_NAME: &'static str = "WriteOnly";
}

impl PortWriteAccess for WriteOnlyAccess {}

#[derive(Debug)]
pub struct ReadWriteAccess(());

impl hidden::Access for ReadWriteAccess {
    const DEBUG_NAME: &'static str = "ReadWrite";
}

impl PortWriteAccess for ReadWriteAccess {}
impl PortReadAccess for ReadWriteAccess {}

/// A port
#[derive(Clone)]
pub struct PortGeneric<T, A> {
    port: u16,
    // phantom data allows type T
    _type: PhantomData<T>,
    // phantom data for acceess
    _access: PhantomData<A>,
}

/// A read-write I/O port.
pub type Port<T> = PortGeneric<T, ReadWriteAccess>;

/// A read-only I/O port.
pub type PortReadOnly<T> = PortGeneric<T, ReadOnlyAccess>;

/// A write-only I/O port.
pub type PortWriteOnly<T> = PortGeneric<T, WriteOnlyAccess>;

impl<T, A> PortGeneric<T, A> {
    pub const fn new(port: u16) -> Self {
        Self {
            port,
            _type: PhantomData,
            _access: PhantomData,
        }
    }

    pub const fn port(&self) -> u16 {
        self.port
    }

    pub const fn size(&self) -> usize {
        core::mem::size_of::<T>()
    }
}

impl<T: PortWrite, A: PortWriteAccess> PortGeneric<T, A> {
    /// # Safety
    /// This cuntion is unsafe
    pub unsafe fn write(&mut self, value: T) {
        T::write(self.port, value);
    }

    /// # Safety
    /// This cuntion is unsafe
    pub unsafe fn writes(&mut self, value: &[T]) {
        T::writes(self.port, value);
    }
}

impl<T: PortRead, A: PortReadAccess> PortGeneric<T, A> {
    /// # Safety
    /// This function is unsafe
    pub unsafe fn read(&self) -> T {
        T::read(self.port)
    }

    /// # Safety
    /// This cuntion is unsafe
    pub unsafe fn reads(&mut self, value: &mut [T]) {
        T::reads(self.port, value);
    }
}

impl<T, A> PartialEq for PortGeneric<T, A> {
    fn eq(&self, other: &Self) -> bool {
        self.port == other.port
    }
}

impl<T, A> Eq for PortGeneric<T, A> {}

impl<T, A: hidden::Access> fmt::Debug for PortGeneric<T, A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Port")
            .field("port", &self.port)
            .field("size", &self.size())
            .field("access", &format_args!("{}", A::DEBUG_NAME))
            .finish()
    }
}

impl PortRead for u8 {
    unsafe fn read(port: u16) -> Self {
        let output: Self;
        asm!("in al, dx", out("al") output, in("dx") port, options(nomem, nostack, preserves_flags));
        output
    }

    unsafe fn reads(port: u16, buf: &mut [Self])
    where
        Self: Sized,
    {
        asm!("rep insb", in("dx") port, in("ecx") buf.len(), in("edi") buf.as_ptr());
    }
}

impl PortWrite for u8 {
    unsafe fn write(port: u16, value: Self) {
        asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack, preserves_flags));
    }

    unsafe fn writes(port: u16, buf: &[Self])
    where
        Self: Sized,
    {
        asm!("rep outsb", in("dx") port, in("ecx") buf.len(), in("esi") buf.as_ptr());
    }
}

impl PortRead for u16 {
    unsafe fn read(port: u16) -> Self {
        let output: Self;
        asm!("in ax, dx", out("ax") output, in("dx") port, options(nomem, nostack, preserves_flags));
        output
    }

    unsafe fn reads(port: u16, buf: &mut [Self])
    where
        Self: Sized,
    {
        asm!("rep insw", in("dx") port, in("ecx") buf.len(), in("edi") buf.as_ptr());
    }
}

impl PortWrite for u16 {
    unsafe fn write(port: u16, value: Self) {
        asm!("out dx, ax", in("dx") port, in("ax") value, options(nomem, nostack, preserves_flags));
    }

    unsafe fn writes(port: u16, buf: &[Self])
    where
        Self: Sized,
    {
        asm!("rep outsw", in("dx") port, in("ecx") buf.len(), in("esi") buf.as_ptr());
    }
}

impl PortRead for u32 {
    unsafe fn read(port: u16) -> Self {
        let output: Self;
        asm!("in eax, dx", out("eax") output, in("dx") port, options(nomem, nostack, preserves_flags));
        output
    }

    unsafe fn reads(port: u16, buf: &mut [Self])
    where
        Self: Sized,
    {
        asm!("rep insd", in("dx") port, in("ecx") buf.len(), in("edi") buf.as_ptr());
    }
}

impl PortWrite for u32 {
    unsafe fn write(port: u16, value: Self) {
        asm!("out dx, eax", in("dx") port, in("eax") value, options(nomem, nostack, preserves_flags));
    }

    unsafe fn writes(port: u16, buf: &[Self])
    where
        Self: Sized,
    {
        asm!("rep outsd", in("dx") port, in("ecx") buf.len(), in("esi") buf.as_ptr());
    }
}
