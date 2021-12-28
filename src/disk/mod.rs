use ide::Ata;
use spin::{Lazy, Mutex};

pub mod ext;
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
    let bpb = unsafe { *(&buf as *const _ as *const fat::BiosParameterBlock) };

    crate::kdbg!(&bpb);
    crate::kdbg!(&bpb.is_fat32());
    crate::kdbg!(&bpb.file_system_type());
    crate::kdbg!(&bpb.volume_label());
    crate::kdbg!(&bpb.oem_name());
    // buf[0] = 0xFFFF;
    // ide.write(0, &buf);
}
