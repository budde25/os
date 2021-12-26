use ide::Ataio;
use spin::{Lazy, Mutex};

pub mod fat;
pub mod ide;

static IDE: Lazy<Mutex<Ataio>> = Lazy::new(|| {
    let ata = Ataio::default();
    Mutex::new(ata)
});

pub fn ide_init() {
    let disk_1 = IDE.lock().init();
    crate::kprintln!("Disk 1 exists: {disk_1}");
}
