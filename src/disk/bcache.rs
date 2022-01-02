use super::buf::Buffer;
use core::cell::RefCell;
use staticvec::StaticVec;

static mut BUFFERS: StaticVec<RefCell<Buffer>, 30> = StaticVec::new();

pub struct BufferCache {
    _private: (),
}

impl BufferCache {
    pub const fn new() -> Self {
        Self { _private: () }
    }

    unsafe fn get(&mut self, device: u32, block_no: u32) -> &'static RefCell<Buffer> {
        // check the cache
        for buf in &BUFFERS {
            if buf.borrow().device() == device && buf.borrow().block_no() == block_no {
                buf.borrow_mut().ref_inc();
                return buf;
            }
        }

        //not cached, add to cache
        let insert_buf = RefCell::new(Buffer::new(device, block_no));
        BUFFERS.push(insert_buf);
        // TODO this could probably be faster
        for buf in &BUFFERS {
            if buf.borrow().device() == device && buf.borrow().block_no() == block_no {
                buf.borrow_mut().ref_inc();
                return buf;
            }
        }

        unreachable!()
    }

    pub fn read(&mut self, device: u32, block_no: u32) -> &'static RefCell<Buffer> {
        let buf = unsafe { self.get(device, block_no) };
        if !buf.borrow().is_valid() {
            super::ide::add_ide_queue(buf);
        }

        buf
    }

    pub fn write(buf: &'static RefCell<Buffer>) {
        buf.borrow_mut().set_dirty(true);
        super::ide::add_ide_queue(buf);
    }
}
