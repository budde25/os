use bitflags::bitflags;
use core::ops::{Index, IndexMut};

use crate::{kdbg, PhysicalAddress, VirtualAddress};

bitflags! {
    struct InterruptCommand: u32 {
        const INIT     =  0x00000500;  // INIT/RESET
        const STARTUP  =  0x00000600;  // Startup IPI
        const DELIVS   =  0x00001000;  // Delivery status
        const ASSERT   =  0x00004000;  // Assert interrupt (vs deassert)
        const DEASSERT =  0x00000000;
        const LEVEL    =  0x00008000;  // Level triggered
        const BCAST    =  0x00080000;  // Send to all APICs, including self.
        const BUSY     =  0x00001000;
        const FIXED    =  0x00000000;
    }
}

bitflags! {
    pub struct ErrorStatus: u32 {
        const SEND_CHECKSUM =          0b00000001;
        const RECIEVE_CHECKSUM =       0b00000010;
        const SEND_ACCEPT =            0b00000100;
        const RECIEVE_ACCEPT =         0b00001000;
        const REDIRECTABLE_IPI =       0b00010000;
        const SEND_ILLIGAL_VECTOR =    0b00100000;
        const RECIEVE_ILLEGAL_VECTOR = 0b01000000;
        const ILLEGAL_REGISTER_ADDR =  0b10000000;
    }
}

#[derive(Debug, Clone, Copy)]
enum Register {
    Id,
    Version,
    TaskPriority,
    ArbitrationPriority,
    ProcessorPriority,
    EndOfInterrupt,
    RemoteRead,
    LogicalDestination,
    DestinationFormat,
    SpuriousInterruptVector,
    InService(u8),
    TriggerMode(u8),
    InterruptRequest(u8),
    ErrorStatus,
    LvtCorrectMachineCheck,
    InterruptCommand(u8),
    // local vector table start
    Timer,
    ThermalSensor,
    PerformanceMonitoring,
    Lint(u8),
    Error,
    // local vector table end
    TimerInitialCount,
    TimerCurrentCount,
    TimerDivideConfiguration,
}

pub struct Lapic(&'static mut Registers);

impl Lapic {
    pub fn new(value: &'static mut Registers) -> Self {
        Self(value)
    }

    pub fn init(&mut self) {
        use crate::interrupts::idt::InterruptIndex;
        use InterruptCommand as ICR;
        use Register as Reg;

        let irq0 = crate::consts::IRQ_0;

        // TODO don't hard code

        // Enable local APIC, set spurious interrupt vector
        let spurious_irq = InterruptIndex::Spurious as u8;
        self.write(
            Reg::SpuriousInterruptVector,
            0x100 | (irq0 + spurious_irq) as u32,
        );

        // Timer counts down at bus frequency
        let timer_irq = InterruptIndex::Timer as u8;
        let x1 = 0xb;
        let periodic = 0x20000;
        self.write(Reg::TimerDivideConfiguration, x1);
        self.write(Reg::Timer, periodic | (irq0 + timer_irq) as u32);
        self.write(Reg::TimerInitialCount, 10_000_000);

        // Disable logical interrupt lines
        let masked = 0x10000;
        self.write(Reg::Lint(0), masked);
        self.write(Reg::Lint(1), masked);

        // Disable performance counter overflow interrupts
        // on machines that proved that interrupt entry
        if (self.read(Reg::Version) >> 16 & 0xFF) >= 4 {
            self.write(Reg::PerformanceMonitoring, masked)
        }

        // map error interrupt
        let error_irq = InterruptIndex::Error as u8;
        self.write(Reg::Error, (irq0 + error_irq) as u32);

        // Ack outstanding interrupts
        self.end_of_interrupt();

        // Send an init level de assert to synchronize arbitration id
        self.write(Reg::InterruptCommand(1), 0);
        self.write(
            Reg::InterruptCommand(0),
            ICR::BCAST.bits | ICR::INIT.bits | ICR::LEVEL.bits,
        );
        while self.read(Reg::InterruptCommand(0)) & ICR::DELIVS.bits != 0 {}

        // Enable interrupt on the APIC (but not the processor)
        self.write(Reg::TaskPriority, 0);
    }

    /// Write to a register
    fn write(&mut self, index: Register, value: u32) {
        self.0[index].write(value);

        // wait for write to finish, by reading
        self.id();
    }

    /// Read from a register
    fn read(&self, index: Register) -> u32 {
        self.0[index].read()
    }

    /// Get the Id of the lapic
    pub fn id(&self) -> u8 {
        (self.read(Register::Id) >> 24) as u8
    }

    pub fn error_status(&self) -> ErrorStatus {
        ErrorStatus::from_bits_truncate(self.read(Register::ErrorStatus))
    }

    /// call when an interrupt has ended
    pub fn end_of_interrupt(&mut self) {
        self.write(Register::EndOfInterrupt, 0);
    }

    /// Start additional processors
    pub fn start_ap(&mut self, apic_id: u8, addr: PhysicalAddress) {
        // "The BSP must initialize CMOS shutdown code to 0AH
        // and the warm reset vector (DWORD based at 40:67) to point at
        // the AP startup code prior to the [universal startup algorithm]."
        unsafe { crate::io::CMOS.shutdown() };

        let wrv_value: u16 = u64::from(addr) as u16;
        let warm_reset_vector = PhysicalAddress::new(0x40 << 4 | 0x67).as_mut_ptr::<u16>();

        unsafe { warm_reset_vector.write_unaligned(wrv_value) };

        use InterruptCommand as IC;
        use Register as Reg;

        // universal startup algorithm
        self.write(Reg::InterruptCommand(1), (apic_id as u32) << 24);
        self.write(
            Reg::InterruptCommand(0),
            IC::INIT.bits | IC::LEVEL.bits | IC::ASSERT.bits,
        );
        super::micro_delay(200);
        assert!(self.error_status().is_empty());

        self.write(Reg::InterruptCommand(0), IC::INIT.bits | IC::LEVEL.bits);

        super::micro_delay(100);
        assert!(self.error_status().is_empty());
        // send ipi twice!
        for _ in 0..2 {
            self.write(Reg::InterruptCommand(1), (apic_id as u32) << 24);
            self.write(
                Reg::InterruptCommand(0),
                IC::STARTUP.bits | (u64::from(addr) >> 12) as u32,
            );

            super::micro_delay(200);
            assert!(self.error_status().is_empty());
        }
    }
}

impl Default for Lapic {
    fn default() -> Self {
        use crate::tables::MADT_TABLE;
        let pa = MADT_TABLE.lapic_addr();
        let ptr = unsafe { &mut *pa.as_mut_ptr::<Registers>() };
        Self::new(ptr)
    }
}

// TODO we must write a custom debug that only reads from the readable registers, else we lock it up
#[repr(C)]
pub struct Registers {
    _reserved_1: [Reg; 2],           // none
    id: Reg,                         // read/write
    version: Reg,                    // read only
    _reserved_2: [Reg; 4],           // none
    task_priority: Reg,              // read/write
    arbitration_priority: Reg,       // read only
    processor_priority: Reg,         // read only
    end_of_interrupt: Reg,           // write only
    remote_read: Reg,                // read only
    logical_destination: Reg,        // read/write
    destination_format: Reg,         // read/write
    spurious_interrupt_vector: Reg,  // read/write
    in_service: [Reg; 8],            // read only
    trigger_mode: [Reg; 8],          // read only
    interrupt_request: [Reg; 8],     // read only
    error_status: Reg,               // read only
    _reserved_3: [Reg; 6],           // none
    lvt_correct_machine_check: Reg,  // read/write
    interrupt_command: [Reg; 2],     // read/write
    lvt_timer: Reg,                  // read/write
    lvt_thermal_sensor: Reg,         // read/write
    lvt_performance_monitoring: Reg, // read/write
    lvt_lint: [Reg; 2],              // read/write
    lvt_error: Reg,                  // read/write
    initial_count: Reg,              // read/write
    current_count: Reg,              // read only
    _reserved_4: [Reg; 4],           // none
    divide_configuration: Reg,       // read/write
    _reserved_5: Reg,                // none
}

impl Index<Register> for Registers {
    type Output = Reg;
    fn index(&self, index: Register) -> &Self::Output {
        match index {
            Register::Id => &self.id,
            Register::Version => &self.version,
            Register::TaskPriority => &self.task_priority,
            Register::ArbitrationPriority => &self.arbitration_priority,
            Register::ProcessorPriority => &self.processor_priority,
            Register::EndOfInterrupt => panic!("End of Interrupt is write only"),
            Register::RemoteRead => &self.remote_read,
            Register::LogicalDestination => &self.logical_destination,
            Register::DestinationFormat => &self.destination_format,
            Register::SpuriousInterruptVector => &self.spurious_interrupt_vector,
            Register::InService(i) if i < 8 => &self.in_service[i as usize],
            Register::InService(_) => panic!("Index out of bounds, must be < 8"),
            Register::TriggerMode(i) if i < 8 => &self.trigger_mode[i as usize],
            Register::TriggerMode(_) => panic!("Index out of bounds, must be < 8"),
            Register::InterruptRequest(i) if i < 8 => &self.interrupt_request[i as usize],
            Register::InterruptRequest(_) => panic!("Index out of bounds, must be < 8"),
            Register::ErrorStatus => &self.error_status,
            Register::LvtCorrectMachineCheck => &self.lvt_correct_machine_check,
            Register::InterruptCommand(i) if i < 2 => &self.interrupt_command[i as usize],
            Register::InterruptCommand(_) => panic!("Index out of bounds, must be < 2"),
            Register::Timer => &self.lvt_timer,
            Register::ThermalSensor => &self.lvt_thermal_sensor,
            Register::PerformanceMonitoring => &self.lvt_performance_monitoring,
            Register::Lint(i) if i < 2 => &self.lvt_lint[i as usize],
            Register::Lint(_) => panic!("Index out of bounds, must be < 2"),
            Register::Error => &self.lvt_error,
            Register::TimerInitialCount => &self.initial_count,
            Register::TimerCurrentCount => &self.current_count,
            Register::TimerDivideConfiguration => &self.divide_configuration,
        }
    }
}

impl IndexMut<Register> for Registers {
    fn index_mut(&mut self, index: Register) -> &mut Self::Output {
        match index {
            Register::Id => &mut self.id,
            Register::Version => panic!("Version is read only"),
            Register::TaskPriority => &mut self.task_priority,
            Register::ArbitrationPriority => panic!("Arbitration Priority is read only"),
            Register::ProcessorPriority => panic!("Processor Priority is read only"),
            Register::EndOfInterrupt => &mut self.end_of_interrupt,
            Register::RemoteRead => panic!("Remote Read is read only"),
            Register::LogicalDestination => &mut self.logical_destination,
            Register::DestinationFormat => &mut self.destination_format,
            Register::SpuriousInterruptVector => &mut self.spurious_interrupt_vector,
            Register::InService(_) => panic!("In Service is read only"),
            Register::TriggerMode(_) => panic!("Trigger Mode is read only"),
            Register::InterruptRequest(_) => panic!("Interrupt Request is read only"),
            Register::ErrorStatus => panic!("Error Status is read only"),
            Register::LvtCorrectMachineCheck => &mut self.lvt_correct_machine_check,
            Register::InterruptCommand(i) if i < 2 => &mut self.interrupt_command[i as usize],
            Register::InterruptCommand(_) => panic!("Index out of bounds, must be < 2"),
            Register::Timer => &mut self.lvt_timer,
            Register::ThermalSensor => &mut self.lvt_thermal_sensor,
            Register::PerformanceMonitoring => &mut self.lvt_performance_monitoring,
            Register::Lint(i) if i < 2 => &mut self.lvt_lint[i as usize],
            Register::Lint(_) => panic!("Index out of bounds, must be < 2"),
            Register::Error => &mut self.lvt_error,
            Register::TimerInitialCount => &mut self.initial_count,
            Register::TimerCurrentCount => panic!("Current Count is read only"),
            Register::TimerDivideConfiguration => &mut self.divide_configuration,
        }
    }
}

/// A u32 register that takes up the space of a u128, required by the lapic, we are expected to only read and write the lower 32 bits
/// This thus cannot implement Copy or Clone as that would read the reserved space
#[repr(C, packed)]
struct Reg {
    _reserved: [u32; 3],
    data: u32,
}

impl Reg {
    /// We are NOT allowed to write to the reserved space, so we must be careful to only write to the lowest 4 bytes (u32)
    pub fn write(&mut self, value: u32) {
        let ptr = self as *mut Self as *mut u32;
        let ptr = unsafe { ptr.add(3) }; // skip the reserved
        unsafe { ptr.write_volatile(value) };
    }

    /// We are NOT allowed to read to the reserved space, so we must be careful to only read from the lowest 4 bytes (u32)
    pub fn read(&self) -> u32 {
        let ptr = self as *const Self as *const u32;
        let ptr = unsafe { ptr.add(3) }; // skip the reserved
        unsafe { ptr.read_volatile() }
    }
}

impl core::fmt::Debug for Reg {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // special care to only read what is allowed
        let data = self.read();
        f.debug_tuple("Register").field(&data).finish()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test_case]
    fn struct_aligned() {
        use core::mem::size_of;
        assert_eq!(size_of::<Registers>(), 16 * 0x40)
    }

    #[test_case]
    fn register_aligned() {
        use core::mem::size_of;
        assert_eq!(size_of::<Reg>(), 16)
    }
}
