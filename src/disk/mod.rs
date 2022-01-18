use bcache::BufferCache;
use core::sync::atomic::AtomicBool;
use ide::Ata;
use spin::Mutex;

pub use ide::interrupt_handler;

mod bcache;
mod buf;
mod ide;

static HAVE_DISK_1: AtomicBool = AtomicBool::new(false);
static IDE: Mutex<Ata> = Mutex::new(Ata::new_primary());
static BUFFERS: Mutex<BufferCache> = Mutex::new(BufferCache::new());

pub fn ide_init() {
    let disk_1 = IDE.lock().init();
    HAVE_DISK_1.store(true, core::sync::atomic::Ordering::Relaxed);
    ide::ide_queue_init();

    crate::kprintln!("Disk 1 exists: {disk_1}");
}

pub fn ide_test() {
    let mut bufs = BUFFERS.lock();
    for i in 0..40 {
        let _data = bufs.read(1, i);
    }
    //*data.borrow_mut().data_mut() = [0x00u8; 1024];
    //bcache::BufferCache::write(data);
}

/// Defines the methods required to achive DiskIO
trait DiskIo<const S: usize> {
    /// Reads from a disk at lba of size 1024
    /// Takes a mutable buffer and reads onto that
    fn read(&mut self, lba: u32, buf: &mut [u8; S]);
    /// Writes to a disk at lba of size 1024
    /// Take an imutable buffer and write from that
    fn write(&mut self, lba: u32, buf: &[u8; S]);
}
