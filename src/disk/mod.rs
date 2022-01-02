use bcache::BufferCache;
use core::sync::atomic::AtomicBool;
use ide::Ata;
use spin::Mutex;

pub mod bcache;
pub mod buf;
pub mod ext;
pub mod fat;
pub mod ide;

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
    let data = bufs.read(1, 0);
    data.borrow_mut().data()[1] = 0xFF;
    bcache::BufferCache::write(data);
}
