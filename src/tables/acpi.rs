use crate::PhysicalAddress;
use core::str;
use core::{fmt::Debug, mem::size_of, ptr::addr_of};

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct RSDPV1 {
    signature: [u8; 8],
    checksum: u8,
    oem_id: [u8; 6],
    revision: u8,
    rsdt_address: u32,
}

impl RSDPV1 {
    pub fn rsdt_address(&self) -> PhysicalAddress {
        PhysicalAddress::new(self.rsdt_address.into())
    }
}

impl Debug for RSDPV1 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let signature = str::from_utf8(&self.signature).unwrap();
        let oem_id = str::from_utf8(&self.oem_id).unwrap();
        f.debug_struct("RESDV1")
            .field("signature", &signature)
            .field("checksum", &self.checksum)
            .field("oem_id", &oem_id)
            .field("revision", &self.revision)
            .field("rsdt_address", &self.rsdt_address())
            .finish()
    }
}

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct RSDPV2 {
    signature: [u8; 8],
    checksum: u8,
    oem_id: [u8; 6],
    revision: u8,
    rsdt_address: u32,
    // only if revision 2
    length: u32,
    xsdt_address: u64,
    extended_checksum: u8,
    _reserved: [u8; 3],
}

impl RSDPV2 {
    pub fn rsdt_address(&self) -> PhysicalAddress {
        PhysicalAddress::new(self.rsdt_address.into())
    }

    pub fn xsdt_address(&self) -> PhysicalAddress {
        PhysicalAddress::new(self.xsdt_address)
    }
}

impl Debug for RSDPV2 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let signature = str::from_utf8(&self.signature).unwrap();
        let oem_id = str::from_utf8(&self.oem_id).unwrap();
        let length = self.length; // fix unaligned ref
        f.debug_struct("RESDV2")
            .field("signature", &signature)
            .field("checksum", &self.checksum)
            .field("oem_id", &oem_id)
            .field("revision", &self.revision)
            .field("rsdt_address", &self.rsdt_address())
            .field("length", &length)
            .field("xsdt_address", &self.xsdt_address())
            .field("extended_checksum", &self.extended_checksum)
            .finish()
    }
}

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct ACPISDTHeader {
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

impl ACPISDTHeader {
    pub fn signature(&self) -> &'static str {
        let ptr = addr_of!(self.signature) as *const u8;
        let slice = unsafe { core::slice::from_raw_parts(ptr, 4) };
        unsafe { core::str::from_utf8_unchecked(slice) }
    }

    pub fn oem_id(&self) -> &'static str {
        let ptr = addr_of!(self.oem_id) as *const u8;
        let slice = unsafe { core::slice::from_raw_parts(ptr, 6) };
        unsafe { core::str::from_utf8_unchecked(slice) }
    }

    pub fn oem_table_id(&self) -> &'static str {
        let ptr = addr_of!(self.oem_table_id) as *const u8;
        let slice = unsafe { core::slice::from_raw_parts(ptr, 6) };
        unsafe { core::str::from_utf8_unchecked(slice) }
    }

    pub fn length(&self) -> u32 {
        self.length
    }

    /// Returns true if the table is valid, false otherswise
    pub fn is_valid(&self) -> u64 {
        let mut sum: u64 = 0;
        let mut ptr = self as *const ACPISDTHeader as *mut u8;
        for _ in 0..self.length {
            sum += unsafe { *ptr } as u64;
            ptr = unsafe { ptr.add(1) };
        }

        sum % 0x100
    }
}

impl Debug for ACPISDTHeader {
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
    header: ACPISDTHeader,
    pointers: [u32; 0],
}

impl Rsdt {
    pub fn total_entries(&self) -> usize {
        (self.header.length as usize - size_of::<ACPISDTHeader>()) / 4
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
            let ptr = self.rsdt.entry(i).as_ptr::<ACPISDTHeader>();
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
    header: ACPISDTHeader,
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
