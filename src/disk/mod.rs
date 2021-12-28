use ide::Ata;
use spin::{Lazy, Mutex};

pub mod fat;
pub mod ide;

static IDE: Lazy<Mutex<Ata>> = Lazy::new(|| {
    let ata = Ata::new_primary();
    Mutex::new(ata)
});

pub fn ide_init() {
    let disk_1 = IDE.lock().init();

    crate::kprintln!("Disk 1 exists: {disk_1}");
}

pub fn ide_test() {
    let mut ide = IDE.lock();
    let mut buf = [0; 256];
    ide.read(0, &mut buf);
    buf[0] = 0xFFFF;
    ide.write(0, &buf);
}
