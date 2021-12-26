use bitflags::bitflags;
use port::{Port, PortReadOnly, PortWriteOnly};

bitflags! {
    struct Command: u8 {
        const NOP = 0x00;
        const CFA_REQUEST_EXTENDED_ERROR_CODE = 0x03;
        const DATA_SET_MANAGEMENT = 0x06;
        const DATA_SET_MANAGEMENT_XL = 0x07;
        const DEVICE_RESET = 0x08;
        const GET_PHYISICAL_ELEMENT_STATUS = 0x12;
        const READ_SECTORS = 0x20;
        const READ_SECTORS_NO_RETRY = 0x21;
        const READ_LONG = 0x22;
        const READ_LONG_NO_RETRY = 0x23;
        const READ_SECTORS_EXT = 0x24;
        const READ_DMA_EXT = 0x25;
        const READ_DMA_QUEUED_EXT = 0x26;
        const READ_NATIVE_MAX_ADDRESS_EXT = 0x27;
        const READ_MULTIPLE_EXT = 0x29;
        const READ_STERAM_DMA_EXT = 0x2A;
        const READ_STERAM_EXT = 0x2B;
        const READ_LONG_EXT = 0x2F;
        const WRITE_SECTORS = 0x30;
        const WRITE_SECTORS_NO_RETRY = 0x31;
        const WRITE_LONG = 0x32;
        const WRITE_LONG_NO_RETRY = 0x33;
        const WRTIRE_SECTORS_EXT = 0x34;
        const WRITE_DMA_EXT = 0x35;
        const WRITE_DMA_QUEUED_EXT = 0x36;
        const SET_MAX_ADDRESS_EXT = 0x37;
        const CFA_WRITE_SECTORS_WITHOUT_ERASE = 0x38;
        const WRITE_MULIPLE_EXT = 0x39;
        const WRITE_STREAM_DMA_EXT = 0x3A;
        const WRITE_STREAM_EXT = 0x3B;
        const WRITE_VERIFY = 0x3C;
        const WRITE_DMA_FUA_EXT = 0x3D;
        const WRITE_DMA_QUEUED_FUA_EXT = 0x3E;
        const WRITE_LOG_EXT = 0x3F;
        const READ_VERIFY_SECTORS = 0x40;
        const READ_VERIFY_SECTORS_NO_RETRY = 0x41;
        const READ_VERIFY_SECTORS_EXT = 0x42;
        const ZERO_EXT = 0x44;
        const WRITE_UNCORRECTABLE_EXT = 0x45;
        const READ_LOG_DMA_EXT = 0x47;
        const ZAC_MANAGEMENT_IN = 0x4A;
        const FORMAT_TRACK = 0x50;
        const CONFIGURE_STREAM = 0x51;
        const WRITE_LOG_DMA_EXT = 0x57;
        const TRUSTED_NON_DATA = 0x5B;
        const TRUSTED_RECEIVE = 0x5C;
        const TRUSTED_RECEIVE_DMA = 0x5D;
        const TRUSTED_SEND = 0x5E;
        const TRUSTED_SEND_DMA = 0x5F;
        const READ_FPDMA_QUEUED = 0x60;
        const WRITE_FPDMA_QUEUED = 0x61;
        const NXQ_NON_DATA = 0x63;
        const SEND_FPDMA_QUEUED = 0x64;
        const RECEIVE_FPDMA_QUEUED = 0x65;

        // TODO Finish matrix https://wiki.osdev.org/ATA_Command_Matrix
    }
}

bitflags! {
    struct ErrorStatus: u8 {
        const NO_ADDRESS_MARK = 0x01;
        const TRACK_0_NOT_FOUND = 0x02;
        const COMMAND_ABORTED = 0x04;
        const MEDIA_CHANGE_REQUEST = 0x08;
        const ID_MARK_NOT_FOUND = 0x10;
        const MEDIA_CHANGED = 0x20;
        const UNCORRECTABLE_DATA = 0x40;
        const BAD_BLOCK = 0x80;
    }
}

bitflags! {
    struct Status: u8 {
        const ERROR = 0x01;
        const INDEX = 0x02;
        const CORRECTED = 0x04;
        const DRQ = 0x08;
        const SRV = 0x10;
        const DRIVE_FAULT = 0x20; //does not set error
        const READY = 0x40;
        const BUSY = 0x80;
    }
}

pub struct Ataio {
    data: Port<u16>,             // index 0
    error: PortReadOnly<u8>,     // index 1
    features: PortWriteOnly<u8>, // index 1
    sector_count: Port<u8>,      // index 2
    sector_number: Port<u8>,     // index 3
    cylinder_low: Port<u8>,      // index 4
    cylinder_high: Port<u8>,     // index 5
    drive_head: Port<u8>,        // index 6
    status: PortReadOnly<u8>,    // index 7
    command: PortWriteOnly<u8>,  // index 7
}

impl Ataio {
    fn new(port: u16) -> Self {
        Self {
            data: Port::new(port),
            error: PortReadOnly::new(port + 1),
            features: PortWriteOnly::new(port + 1),
            sector_count: Port::new(port + 2),
            sector_number: Port::new(port + 3),
            cylinder_low: Port::new(port + 4),
            cylinder_high: Port::new(port + 5),
            drive_head: Port::new(port + 6),
            status: PortReadOnly::new(port + 7),
            command: PortWriteOnly::new(port + 7),
        }
    }

    pub fn init(&mut self) -> bool {
        use crate::consts::IRQ;

        crate::io::IO_APIC.lock().enable(IRQ::Ide, 0);
        self.wait();

        let mut have_disk_1 = false;

        unsafe { self.drive_head.write(0xe0 | (1 << 4)) };
        for _ in 0..1000 {
            if unsafe { self.status.read() } != 0 {
                have_disk_1 = true;
            }
        }

        unsafe { self.drive_head.write(0xe0 | (0 << 4)) };

        have_disk_1
    }

    fn error(&self) -> ErrorStatus {
        unsafe { ErrorStatus::from_bits_truncate(self.error.read()) }
    }

    fn status(&self) -> Status {
        unsafe { Status::from_bits_truncate(self.status.read()) }
    }

    /// wait for ide disk to be ready
    fn wait(&self) {
        const IDE_DRDY: u8 = 0x40;
        // FIXME: Will spin forever if disk is not present
        loop {
            let stat = self.status();
            if stat.bits() & (Status::BUSY.bits() | IDE_DRDY) == IDE_DRDY {
                break;
            }
        }
    }
}

impl Default for Ataio {
    fn default() -> Self {
        Self::new(0x1F0)
    }
}
