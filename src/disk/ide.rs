use bitflags::bitflags;
use port::{Port, PortReadOnly, PortWriteOnly};

struct IdeDevice {
    _reserved: u8,     // 0 (Empty) or 1 (This Drive really exists).
    channel: u8,       // 0 (Primary Channel) or 1 (Secondary Channel).
    drive: u8,         // 0 (Master Drive) or 1 (Slave Drive).
    r#type: u16,       // 0: ATA, 1:ATAPI.
    signature: u16,    // Drive Signature
    capabilities: u16, // Features.
    command_sets: u32, // Command Sets Supported.
    size: u32,         // Size in Sectors.
    model: [u8; 41],   // Model in cstring.
}

bitflags! {
    struct Command: u8 {
        const READ_PIO        = 0x20;
        const READ_PIO_EXT    = 0x24;
        const READ_DMA        = 0xC8;
        const READ_DMA_EXT    = 0x25;
        const WRITE_PIO       = 0x30;
        const WRITE_PIO_EXT   = 0x34;
        const WRITE_DMA       = 0xCA;
        const WRITE_DMA_EXT   = 0x35;
        const CACHE_FLUSH     = 0xE7;
        const CACHE_FLUSH_EXT = 0xEA;
        const PACKET          = 0xA0;
        const IDENTIFY_PACKET = 0xA1;
        const IDENTIFY        = 0xEC;
    }
}

bitflags! {
    struct Errors: u8 {
        const AMNF  = 0x01; // No address mark
        const TK0NF = 0x02; // Track 0 not found
        const ABRT  = 0x04; // Command Aborted
        const MCR   = 0x08; // Media change request
        const IDNF  = 0x10; // ID mark not found
        const MC    = 0x20; // Media changed
        const UNC   = 0x40; // Unccorrectable data
        const BBK   = 0x80; // Bad block
    }
}

bitflags! {
    struct Status: u8 {
        const ERR  = 0x01; // Error
        const IDX  = 0x02; // Index
        const CORR = 0x04; // Corrected data
        const DRQ  = 0x08; // Data request ready
        const DSC  = 0x10; // Drive seek complete
        const DF   = 0x20; // Drive write fault (does not set error)
        const DRDY = 0x40; // Drive ready
        const BSY  = 0x80; // Busy
    }
}

bitflags! {
    struct Identify: u8 {
        const DEVICETYPE   = 0;
        const CYLINDERS    = 2;
        const HEADS        = 6;
        const SECTORS      = 12;
        const SERIAL       = 20;
        const MODEL        = 54;
        const CAPABILITIES = 98;
        const FIELDVALID   = 106;
        const MAX_LBA      = 120;
        const COMMANDSETS  = 164;
        const MAX_LBA_EXT  = 200;
    }
}

bitflags! {
    struct DriveHead: u8 {
        const SLAVE      = 0x10; // slave bit. 0 master, 1 slave
        const RESERVED_1 = 0x20; // should be set
        const LBA        = 0x40; // 0 chs, 1 lba
        const RESERVED_2 = 0x80; // should be set
    }
}

pub struct Ata {
    primary: bool,                      // primary if true, seconday if false
    data: Port<u16>,                    // index 0
    error: PortReadOnly<u8>,            // index 1
    features: PortWriteOnly<u8>,        // index 1
    sector_count: Port<u8>,             // index 2
    sector_number: Port<u8>,            // index 3 | LBA0
    cylinder_low: Port<u8>,             // index 4 | LBA1
    cylinder_high: Port<u8>,            // index 5 | LBA2
    drive_head: Port<u8>,               // index 6
    status: PortReadOnly<u8>,           // index 7
    command: PortWriteOnly<u8>,         // index 7
    alternate_status: PortReadOnly<u8>, // index control 2
    control: PortWriteOnly<u8>,         // index control 2
}

impl Ata {
    fn new(io_port: u16, control_port: u16, primary: bool) -> Self {
        Self {
            primary,
            data: Port::new(io_port),
            error: PortReadOnly::new(io_port + 1),
            features: PortWriteOnly::new(io_port + 1),
            sector_count: Port::new(io_port + 2),
            sector_number: Port::new(io_port + 3),
            cylinder_low: Port::new(io_port + 4),
            cylinder_high: Port::new(io_port + 5),
            drive_head: Port::new(io_port + 6),
            status: PortReadOnly::new(io_port + 7),
            command: PortWriteOnly::new(io_port + 7),
            // control port
            alternate_status: PortReadOnly::new(control_port + 2),
            control: PortWriteOnly::new(control_port + 2),
        }
    }

    pub fn new_primary() -> Self {
        Self::new(0x1f0, 0x3f6, true)
    }

    pub fn new_secondary() -> Self {
        Self::new(0x170, 0x376, false)
    }

    pub fn init(&mut self) -> bool {
        use crate::consts::IRQ;

        unsafe { (*crate::io::IO_APIC.as_mut_ptr()).enable(IRQ::Ide, 0) };
        self.poll(false).unwrap();

        let mut have_disk_1 = false;

        unsafe { self.drive_head.write(0xe0 | (1 << 4)) };
        for _ in 0..1000 {
            if unsafe { self.status.read() } != 0 {
                have_disk_1 = true;
            }
        }

        // switch to disk 0
        // unsafe { self.drive_head.write(0xe0 | (0 << 4)) };

        have_disk_1
    }

    fn error(&self) -> Errors {
        unsafe { Errors::from_bits_truncate(self.error.read()) }
    }

    fn status(&self) -> Status {
        unsafe { Status::from_bits_truncate(self.status.read()) }
    }

    pub fn read(&mut self, lba: u32, buf: &mut [u16; 256]) {
        self.setup_access(0, lba).unwrap();
        self._read(buf).unwrap();
    }

    fn _read(&mut self, buf: &mut [u16; 256]) -> Result<(), u64> {
        let num_sects = 1; // TODO: allow for bigger reads in the future
        for _ in 0..num_sects {
            self.poll(true)?;
            unsafe { self.data.reads(buf) };
        }
        unsafe { self.command.write(Command::CACHE_FLUSH.bits()) };
        self.poll(false)?;

        Ok(())
    }

    pub fn write(&mut self, lba: u32, buf: &[u16; 256]) {
        self.setup_access(1, lba).unwrap();
        self._write(buf).unwrap();
    }

    fn _write(&mut self, buf: &[u16; 256]) -> Result<(), u64> {
        let num_sects = 1; // TODO: allow for bigger reads in the future
        for _ in 0..num_sects {
            self.poll(false)?;
            unsafe { self.data.writes(buf) };
        }
        unsafe { self.command.write(Command::CACHE_FLUSH.bits()) };
        self.poll(false)?;

        Ok(())
    }

    fn setup_access(&mut self, direction: u8, lba: u32) -> Result<(), u64> {
        let lba_mode: u8; // 0: CHS, 1: LBA28, 2: LBA48
        let mut lba_io: [u8; 6] = [0; 6];
        let slavebit = 1; // 0 master, 1 slave
        let head: u8;
        let num_sects = 1;

        // disable iqs
        self.disable_irq();

        // select lba mode
        if lba >= 0x10000000 {
            // TODO: support lba48
            panic!("TODO support lba48")
        } else {
            // lba28
            lba_mode = 1;
            lba_io[0] = (lba & 0x00000FF) as u8;
            lba_io[1] = ((lba & 0x000FF00) >> 8) as u8;
            lba_io[2] = ((lba & 0x0FF0000) >> 16) as u8;
            lba_io[3] = 0; // These Registers are not used here.
            lba_io[4] = 0; // These Registers are not used here.
            lba_io[5] = 0; // These Registers are not used here.
            head = ((lba & 0xF000000) >> 24) as u8;
        }
        // TODO: support chs

        let dma = 0; // TODO: support DMA

        // Wait if busy
        while self.status().contains(Status::BSY) {}

        // select drive
        if lba_mode == 0 {
            // drive and chs
            unsafe { self.drive_head.write(0xA0 | (slavebit << 4) | head) };
        } else {
            // drive and lba
            unsafe { self.drive_head.write(0xE0 | (slavebit << 4) | head) };
        }

        // write params
        unsafe {
            if lba_mode == 2 {
                self.sector_count.write(0);
                // self.write(ATA_REG_LBA3, lba_io[3]);
                // self.write(ATA_REG_LBA4, lba_io[4]);
                // self.write(ATA_REG_LBA5, lba_io[5]);
            }
            self.sector_count.write(num_sects);
            self.sector_number.write(lba_io[0]);
            self.cylinder_low.write(lba_io[1]);
            self.cylinder_high.write(lba_io[2]);
        }

        // Routine that is followed:
        // If ( DMA & LBA48)   DO_DMA_EXT;
        // If ( DMA & LBA28)   DO_DMA_LBA;
        // If ( DMA & LBA28)   DO_DMA_CHS;
        // If (!DMA & LBA48)   DO_PIO_EXT;
        // If (!DMA & LBA28)   DO_PIO_LBA;
        // If (!DMA & !LBA#)   DO_PIO_CHS;
        let cmd = match (lba_mode, dma, direction) {
            (0, 0, 0) => Command::READ_PIO,
            (1, 0, 0) => Command::READ_PIO,
            (2, 0, 0) => Command::READ_PIO_EXT,
            (0, 1, 0) => Command::READ_DMA,
            (1, 1, 0) => Command::READ_DMA,
            (2, 1, 0) => Command::READ_DMA_EXT,
            (0, 0, 1) => Command::WRITE_PIO,
            (1, 0, 1) => Command::WRITE_PIO,
            (2, 0, 1) => Command::WRITE_PIO_EXT,
            (0, 1, 1) => Command::WRITE_DMA,
            (1, 1, 1) => Command::WRITE_DMA,
            (2, 1, 1) => Command::WRITE_DMA_EXT,
            _ => unreachable!(),
        };
        unsafe { self.command.write(cmd.bits()) };

        Ok(())
    }

    pub fn disable_irq(&mut self) {
        unsafe { self.control.write(2) };
    }

    fn enable_irq(&mut self) {}

    // TODO proper errors handling
    fn poll(&mut self, advanced: bool) -> Result<(), u64> {
        // delay for 400 nanoseconds
        for _ in 0..4 {
            unsafe { self.alternate_status.read() }; // wastes 100ns
        }

        // wait for BSY to be cleared
        while self.status().contains(Status::BSY) {}

        if !advanced {
            return Ok(());
        }

        let status = self.status();
        if status.contains(Status::ERR) {
            return Err(2);
        }

        if status.contains(Status::DF) {
            return Err(1);
        }

        // BSY = 0; DF = 0; ERR = 0 so we should check for DRQ now
        if !status.contains(Status::DRQ) {
            return Err(3); // DRQ should be set
        }

        Ok(())
    }
}

// static IDE_QUEUE: OnceCell<ArrayQueue<*const RefCell<Buffer>>> = OnceCell::uninit();
// static WAKER: AtomicWaker = AtomicWaker::new();

// pub async fn page_handler() {
//     let mut buffers = BufferSteam::new();
//     // TODO: check to muck sure we have a lock on it

//     while let Some(buf) = buffers.next().await {
//         // read data if needed
//     }
// }
