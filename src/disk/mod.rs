use bcache::BufferCache;
use ide::Ata;
use spin::{Lazy, Mutex};

pub mod bcache;
pub mod buf;
pub mod ext;
pub mod fat;
pub mod ide;

static IDE: Lazy<Mutex<Ata>> = Lazy::new(|| {
    let ata = Ata::new_primary();
    Mutex::new(ata)
});

static BUFFERS: Lazy<Mutex<BufferCache>> = Lazy::new(|| {
    let buffer = BufferCache::new();
    Mutex::new(buffer)
});

pub fn ide_init() {
    let disk_1 = IDE.lock().init();

    crate::kprintln!("Disk 1 exists: {disk_1}");
}

pub fn ide_test() {
    let mut ide = IDE.lock();

    let mut buf = [0; 256];

    ide.read(2, &mut buf);

    let _ext = unsafe { *(&buf as *const _ as *const ext::Superblock) };

    //crate::kdbg!(&ext);
    //crate::kdbg!(ext.verify());
    //crate::kdbg!(ext.volume_name());
    // buf[0] = 0xFFFF;
    // ide.write(0, &buf);
}
