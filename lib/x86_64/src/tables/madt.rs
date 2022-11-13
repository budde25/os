use super::acpi::AcpiSdtHeader;
use crate::PhysicalAddress;
use derive_more::From;

pub struct MultiAPIC {
    start_address: PhysicalAddress,
    madt: &'static Madt,
    index: u32,
    init: bool,
    // these are what we want
    apic_ids: [Option<u8>; 256],
    num_cores: u8,
    lapic_addr: PhysicalAddress,
    ioapic_addr: PhysicalAddress,
}

impl MultiAPIC {
    pub fn new(address: PhysicalAddress) -> Self {
        Self {
            start_address: address,
            madt: unsafe { &*address.as_ptr::<Madt>() },
            index: 0,
            init: false,
            apic_ids: [None; 256],
            num_cores: 0,
            lapic_addr: PhysicalAddress::new(0),
            ioapic_addr: PhysicalAddress::new(0),
        }
    }

    pub fn init(&mut self) {
        use core::mem::size_of;

        if self.init {
            panic!("Already init!")
        }

        let entry_header_size = size_of::<MadtEntryHeader>();

        self.lapic_addr = self.madt.lapic_addr.into();
        self.index = size_of::<AcpiSdtHeader>() as u32 + 8;

        let start = self.start_address;
        while self.index < self.madt.header.length() {
            let header_addr = start + self.index as u64;
            let header = unsafe { *header_addr.as_ptr::<MadtEntryHeader>() };
            let item_size = (header.length - entry_header_size as u8) as u32; // minus 2 since its the size of the tag
            let item = start + self.index as u64;
            self.index += entry_header_size as u32; // increment the tag size

            match header.r#type {
                MadtEntryType::Lapic => {
                    let lapic_entry = unsafe { *item.as_ptr::<LapicEntry>() };
                    self.apic_ids[self.num_cores as usize] = Some(lapic_entry.acpi_id);
                    self.num_cores += 1;
                }
                MadtEntryType::Ioapic => {
                    let ioapic_entry = unsafe { *item.as_ptr::<IoapicEntry>() };
                    self.ioapic_addr = ioapic_entry.address();
                }
                MadtEntryType::IoapicIntSrcOverride => {}
                MadtEntryType::IoapicNonMaskIntSrc => {}
                MadtEntryType::LapicNonMaskInts => {}
                MadtEntryType::LapicAddrOveride => {
                    let lapic_overide = unsafe { *item.as_ptr::<LapicAddrOverrideEntry>() };
                    self.lapic_addr = lapic_overide.address();
                }
                MadtEntryType::ProcLocalx2Apic => {}
            }

            self.index += item_size;
        }

        self.init = true
    }
}

impl MultiAPIC {
    pub fn apic_ids(&self) -> &[Option<u8>] {
        &self.apic_ids
    }

    pub fn num_cores(&self) -> u8 {
        self.num_cores
    }

    pub fn lapic_addr(&self) -> PhysicalAddress {
        self.lapic_addr
    }

    pub fn ioapic_addr(&self) -> PhysicalAddress {
        self.ioapic_addr
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct Madt {
    header: AcpiSdtHeader,
    lapic_addr: u32,
    flags: u32,
}

impl Madt {
    pub fn lapic(&self) -> PhysicalAddress {
        PhysicalAddress::new(self.lapic_addr.into())
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct MadtEntryHeader {
    r#type: MadtEntryType,
    length: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, From)]
#[repr(u8)]
pub enum MadtEntryType {
    Lapic = 0,
    Ioapic = 1,
    IoapicIntSrcOverride = 2,
    IoapicNonMaskIntSrc = 3,
    LapicNonMaskInts = 4,
    LapicAddrOveride = 5,
    ProcLocalx2Apic = 9,
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct LapicEntry {
    header: MadtEntryHeader,
    acpi_processor_id: u8,
    acpi_id: u8,
    flags: u32,
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct IoapicEntry {
    header: MadtEntryHeader,
    io_apic_id: u8,
    _reserved: u8,
    io_apic_address: u32,
    global_system_interupts_base: u32,
}

impl IoapicEntry {
    pub fn address(&self) -> PhysicalAddress {
        self.io_apic_address.into()
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct IoapicIntSrcOverrideEntry {
    header: MadtEntryHeader,
    bus_source: u8,
    irq_source: u8,
    global_system_interrupt: u32,
    flags: u16,
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct IoapicNonMaskIntSrcEntry {
    header: MadtEntryHeader,
    nmi_source: u8,
    _reserved: u8,
    flags: u16,
    global_system_interrupt: u32,
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct LapicNonMaskIntsEntry {
    header: MadtEntryHeader,
    processor_id: u8,
    flags: u16,
    lint: u8,
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct LapicAddrOverrideEntry {
    header: MadtEntryHeader,
    _reserved: u16,
    loapic_addr: u64,
}

impl LapicAddrOverrideEntry {
    pub fn address(&self) -> PhysicalAddress {
        PhysicalAddress::new(self.loapic_addr)
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct ProcLocalx2ApicEntry {
    header: MadtEntryHeader,
    _reserved: u16,
    id: u32,
    flags: u32,
    acpi_id: u32,
}
