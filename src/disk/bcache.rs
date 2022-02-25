use core::cell::RefCell;

use super::buf::Buffer;
use alloc::sync::Arc;

//static mut BUFFERS: StaticVec<RefCell<Buffer>, 30> = StaticVec::new();

pub struct BufferCache {
    buffers: [Option<(Arc<RefCell<Buffer>>, bool)>; 30],
    index: usize,
}

unsafe impl Send for BufferCache {}

impl BufferCache {
    pub const fn new() -> Self {
        const VALUE: Option<(Arc<RefCell<Buffer>>, bool)> = None;
        let buffers = [VALUE; 30];
        Self { buffers, index: 0 }
    }

    fn capacity(&self) -> usize {
        self.buffers.len()
    }

    /// clock algo
    unsafe fn get(&mut self, device: u32, block_no: u32) -> Arc<RefCell<Buffer>> {
        // check the cache
        for opt in &self.buffers {
            if let Some(tup) = opt {
                if tup.0.borrow().device() == device && tup.0.borrow().block_no() == block_no {
                    return tup.0.clone();
                }
            }
        }

        //not cached, add to cache
        let new_buf = (Arc::new(RefCell::new(Buffer::new(device, block_no))), true);
        let mut assure = 0; // TODO: remove once we are sure that this works as intended
        while assure < self.capacity() * 3 {
            self.index = (self.index + 1) % self.capacity();
            if self.buffers[self.index].is_none() {
                self.buffers[self.index] = Some(new_buf.clone());
                return new_buf.0;
            }

            if let Some(tup) = &mut self.buffers[self.index] {
                // we can replace
                if !tup.1 && !tup.0.borrow().is_dirty() {
                    self.buffers[self.index] = Some(new_buf.clone());
                    return new_buf.0;
                } else {
                    tup.1 = false; // clock algo emulates lru
                }
            }
            assure += 1;
        }
        panic!("are we forgeting to flush pages?")
    }

    pub fn read(&mut self, device: u32, block_no: u32) -> Arc<RefCell<Buffer>> {
        let buf = unsafe { self.get(device, block_no) };
        if !buf.borrow().is_valid() {
            super::ide::add_ide_queue(buf.clone());
        }
        buf
    }

    pub fn write(buf: Arc<RefCell<Buffer>>) {
        let buf = buf;

        buf.borrow_mut().set_dirty(true);
        super::ide::add_ide_queue(buf);
    }
}
