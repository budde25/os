use core::fmt::Debug;
use core::ptr::addr_of;
use core::slice;

use crate::PhysicalAddress;

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct AcpiSdtHeader {
    signature: [u8; 4],
    length: u32,
    revision: u8,
    checksum: u8,
    oem_id: [u8; 6],
    oem_table_id: [u8; 8],
    oem_revision: u32,
    creator_id: u32,
    creator_revision: u32,
}

impl AcpiSdtHeader {
    pub fn signature(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.signature) }
    }

    pub fn oem_id(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.oem_id) }
    }

    pub fn oem_table_id(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.oem_table_id) }
    }

    pub fn length(&self) -> u32 {
        self.length
    }

    /// Validation using the checksum
    pub fn is_valid(&self) -> bool {
        let bytes =
            unsafe { slice::from_raw_parts(self as *const _ as *const u8, self.length as usize) };
        bytes.iter().fold(0u8, |acc, val| acc.wrapping_add(*val)) == 0
    }
}

impl Debug for AcpiSdtHeader {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let length = self.length;
        let revision = self.revision;
        let oem_revision = self.oem_revision;
        let creator_id = self.creator_id;
        let creator_revision = self.creator_revision;
        f.debug_struct("ACPISDTHeader")
            .field("signature", &self.signature())
            .field("length", &length)
            .field("revision", &revision)
            .field("checksum_valid", &self.is_valid())
            .field("oem_id", &self.oem_id())
            .field("oem_table_id", &self.oem_table_id())
            .field("oem_revision", &oem_revision)
            .field("creator_id", &creator_id)
            .field("creator_revision", &creator_revision)
            .finish()
    }
}

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct Rsdt {
    header: AcpiSdtHeader,
    pointers: [u32; 0],
}

impl Rsdt {
    pub fn total_entries(&self) -> usize {
        (self.header.length as usize - core::mem::size_of::<AcpiSdtHeader>()) / 4
    }

    pub fn entry(&self, index: usize) -> PhysicalAddress {
        let num_entries = self.total_entries();
        assert!(index < num_entries);

        let mut ptr = addr_of!(self.pointers) as *const u32;
        ptr = unsafe { ptr.add(index) };
        PhysicalAddress::new(unsafe { ptr.read_unaligned().into() })
    }
}

impl Debug for Rsdt {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("RSDT")
            .field("header", &self.header)
            .finish_non_exhaustive()
    }
}

#[derive(Debug)]
pub struct Acpi {
    pub rsdt: &'static Rsdt,
    pub fadt: Option<&'static Fadt>,
    pub madt_ptr: Option<PhysicalAddress>,
    //pub hpet: Option<&'static HPET>,
    //pub WAET: Option<&'static WAET>,
}

impl Acpi {
    pub fn new(rsdt_ptr: PhysicalAddress) -> Self {
        Self {
            rsdt: unsafe { &*rsdt_ptr.as_ptr::<Rsdt>() },
            fadt: None,
            madt_ptr: None,
        }
    }

    pub fn init(&mut self) {
        let entry_count = self.rsdt.total_entries();
        for i in 0..entry_count {
            // TODO parse all tables
            let ptr = self.rsdt.entry(i).as_ptr::<AcpiSdtHeader>();
            let header = unsafe { *ptr };
            //kdbg!(header);
            let signature = header.signature();
            match signature {
                "FACP" => self.fadt = Some(unsafe { &*self.rsdt.entry(i).as_ptr::<Fadt>() }),
                "APIC" => self.madt_ptr = Some(self.rsdt.entry(i)),
                "HPET" => {}
                "WAET" => {}
                _ => todo!(),
            }
        }
    }
}

//spec version 1.0 - 5.2.5 https://uefi.org/sites/default/files/resources/ACPI_1.pdf
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct Fadt {
    header: AcpiSdtHeader,
    firmware_ctrl: u32,
    dsdt: u32,

    // field used in ACPI 1.0; no longer in use, for compatibility only
    int_model: u8,

    preferred_power_management_profile: u8,
    sci_interrupt: u16,
    smi_command_port: u32,
    acpi_enable: u8,
    acpi_disable: u8,
    s4_bios_req: u8,
    pstate_control: u8,
    pm1a_event_block: u32,
    pm1b_event_block: u32,
    pm1a_control_block: u32,
    pm1b_control_block: u32,
    pm2_control_block: u32,
    pmtimer_block: u32,
    gpe0_block: u32,
    gpe1_block: u32,
    pm1_event_length: u8,
    pm1_control_length: u8,
    pm2_control_length: u8,
    pmtimer_length: u8,
    gpe0_length: u8,
    gpe1_length: u8,
    gpe1_base: u8,
    cstate_control: u8,
    worst_c2_latency: u16,
    worst_c3_latency: u16,
    flush_size: u16,
    flush_stride: u16,
    duty_offset: u8,
    duty_width: u8,
    day_alarm: u8,
    month_alarm: u8,
    century: u8,
    reserved_2: u16,
    reserved_3: u8,
    flags: u32,
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
struct GenericAddressStructure {
    address_space: u8,
    bit_width: u8,
    bit_offset: u8,
    access_size: u8,
    address: u64,
}
