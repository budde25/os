use crate::consts::IRQ;
use volatile::Volatile;

pub struct IoApic(Volatile<&'static mut IoApicRegister>);

impl IoApic {
    unsafe fn new(ptr: *mut IoApicRegister) -> Self {
        Self(Volatile::new(&mut *ptr))
    }

    /// Initialize the IO Apic
    pub fn init(&mut self) {
        let maxintr = (self.read(0x1) >> 16) & 0xFF;
        let _id = self.read(0x00) >> 24;

        // TODO make sure it is the right id
        // TODO make this rusty

        // Mark all interrupts edge-triggered, active high, disabled,
        // and not routed to any CPUs.
        for i in 0..maxintr {
            self.write(0x10 + 2 * i, 0x10000 | (32 + i));
            self.write(0x10 + 2 * i + 1, 0);
        }
    }

    pub fn enable(&mut self, irq: IRQ, cpu_num: u32) {
        let irq = usize::from(irq) as u32;
        // Mark interrupt edge-triggered, active high,
        // enabled, and routed to the given cpunum,
        // which happens to be that cpu's APIC ID.
        self.write(0x10 + 2 * irq, 32 + irq);
        self.write(0x10 + 2 * irq + 1, cpu_num << 24);
    }

    /// IO apic id
    pub fn id(&mut self) -> u8 {
        // only want bytes 24 -27
        (self.read(0) >> 24) as u8 & 0b00000111
    }

    fn write(&mut self, register: u32, data: u32) {
        self.0.map_mut(|apic| &mut apic.register).write(register);
        self.0.map_mut(|apic| &mut apic.data).write(data);
    }

    fn read(&mut self, register: u32) -> u32 {
        self.0.map_mut(|apic| &mut apic.register).write(register);
        self.0.map_mut(|apic| &mut apic.data).read()
    }
}

impl Default for IoApic {
    fn default() -> Self {
        use crate::multiboot::MADT_TABLE;
        let addr = MADT_TABLE.ioapic_addr();
        unsafe { Self::new(addr.as_mut_ptr::<IoApicRegister>()) }
    }
}

struct IoApicRegister {
    register: u32,
    _reserved: [u32; 3],
    data: u32,
}

impl IoApicRegister {
    fn set_register(&mut self, register: u32) {
        self.register = register;
    }

    fn write(&mut self, register: u32, data: u32) {
        self.register = register;
        self.data = data;
    }
}
