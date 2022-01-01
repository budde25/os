pub mod cpu;

use cpu::Cpu;
use staticvec::StaticVec;

use crate::PhysicalAddress;

static CPUS: StaticVec<Cpu, 8> = StaticVec::new();

pub fn ap_startup() {
    use crate::paging::allocator::Allocator;
    use crate::tables::MADT_TABLE;

    let _aps_running = 0;

    let num_cores = MADT_TABLE.num_cores();
    let lapic_ids = MADT_TABLE.apic_ids();
    let code = PhysicalAddress::new(0x8000);

    copy_boot_to_addr(code);

    for i in 0..num_cores {
        let _ = Cpu::current_cpu();

        // TODO this will only allow for 2 cpus
        let lapic_id = lapic_ids[i as usize].unwrap();
        if lapic_id == 0 {
            continue;
        }

        let stack = crate::paging::MAPPER.lock().allocate_frame().unwrap();
        let stack_addr = u64::from(stack.address());
        let code_ptr = code.as_mut_ptr::<u64>();
        unsafe { code_ptr.sub(1).write_volatile(stack_addr + 4096) };
        unsafe {
            code_ptr
                .sub(2)
                .write_volatile(mp_enter as *const u64 as u64)
        };

        unsafe { (*crate::io::LAPIC.as_mut_ptr()).start_ap(lapic_id, code) };
    }
}

fn copy_boot_to_addr(addr: PhysicalAddress) {
    // move the code
    extern "C" {
        static __mp_boot_start: usize;
        static __mp_boot_end: usize;
    }
    let mp_boot_start = unsafe { &__mp_boot_start as *const _ as *const u8 };
    let mp_boot_size =
        unsafe { &__mp_boot_end as *const _ as usize - &__mp_boot_start as *const _ as usize };

    let src = PhysicalAddress::new(mp_boot_start as u64);

    unsafe { crate::memory::mem_copy(addr.as_mut_ptr::<u8>(), src.as_ptr::<u8>(), mp_boot_size) };
}

#[no_mangle]
extern "C" fn mp_enter() {
    crate::kdbg!("entered");
}
