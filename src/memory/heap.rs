use crate::consts::{HEAP_SIZE, HEAP_START};
use block_alloc::Allocator;

#[global_allocator]
static ALLOCATOR: Allocator = Allocator::new();

pub fn init() {
    unsafe { ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE) };
    crate::kprintln!(
        "Kernel heap initialized at {:#X}, of size {:#X}",
        HEAP_START,
        HEAP_SIZE
    )
}
