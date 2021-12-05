use block_alloc::Allocator;

pub const HEAP_START: u64 = 0x1000000;
pub const HEAP_SIZE: u64 = 100 * 1024; // 100 KiB

#[global_allocator]
static ALLOCATOR: Allocator = Allocator::new();

pub fn init() {
    unsafe { ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE) };
}
