use super::buf::Buffer;
use core::cell::RefCell;

pub struct BufferCache {
    buffers: [RefCell<Option<Buffer>>; 30],
}

impl BufferCache {
    pub fn new() -> Self {
        const NONE: RefCell<Option<Buffer>> = RefCell::new(None);
        Self {
            buffers: [NONE; 30],
        }
    }

    fn get(&'static mut self, device: u32, block_no: u32) -> &'static RefCell<Option<Buffer>> {
        // check the cache

        for i in 0..self.buffers.len() {
            let buf_ref = self.buffers[i].borrow();
            if let Some(buf) = buf_ref.as_ref() {
                if buf.device() == device && buf.block_no() == block_no {
                    self.buffers[i].borrow_mut().as_mut().unwrap().ref_inc();
                    return &self.buffers[i];
                }
            }
        }

        // not cached, add to cache
        for i in 0..self.buffers.len() {
            if self.buffers[i].borrow().is_some() {
                continue;
            } else {
                *self.buffers[i].borrow_mut() = Some(Buffer::new(device, block_no));
                return &self.buffers[i];
            }
        }

        panic!("buffer full");
    }

    pub fn read(device: u32, block_no: u32) {}
}
