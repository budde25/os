use volatile::Volatile;

struct IOApicRef(Volatile<&'static mut IOApic>);

impl IOApicRef {
    unsafe fn new(ptr: *const u8) -> Self {
        Self(Volatile::new(&mut *(ptr as *mut IOApic)))
    }

    pub fn init(&mut self) {
        let maxintr = (self.read(0x1) >> 16) & 0xFF;
        let id = self.read(0x00) >> 24;
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

impl Default for IOApicRef {
    fn default() -> Self {
        unsafe { Self::new(0xFEC00000 as *mut u8) }
    }
}

struct IOApic {
    register: u32,
    _reserved: [u32; 3],
    data: u32,
}

impl IOApic {
    fn set_register(&mut self, register: u32) {
        self.register = register;
    }

    fn write(&mut self, register: u32, data: u32) {
        self.register = register;
        self.data = data;
    }
}
