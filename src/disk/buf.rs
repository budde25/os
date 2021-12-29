use crate::consts::BSIZE;

#[derive(Debug)]
pub struct Buffer {
    flags: Flags,
    device: u32,
    block_no: u32,
    ref_cnt: u32,
    data: [u8; BSIZE],
}

impl Buffer {
    pub fn new(device: u32, block_no: u32) -> Self {
        Self {
            flags: Flags::empty(),
            device,
            block_no,
            ref_cnt: 1,
            data: [0; BSIZE],
        }
    }

    pub fn device(&self) -> u32 {
        self.device
    }

    pub fn block_no(&self) -> u32 {
        self.block_no
    }

    pub fn ref_count(&self) -> u32 {
        self.ref_cnt
    }

    pub fn ref_inc(&mut self) {
        self.ref_cnt += 1;
    }

    pub fn ref_dec(&mut self) {
        self.ref_cnt -= 1;
    }
}

bitflags::bitflags! {
    pub struct Flags: u32 {
        const VALID = 0x2; // buffer has been read from disk
        const DIRTY = 0x4; // needs to be writen to disk
    }
}
