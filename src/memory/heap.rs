use core::alloc::GlobalAlloc;

use crate::consts::{HEAP_SIZE, HEAP_START};
use block_alloc::Allocator as BlockAllocator;

#[global_allocator]
static ALLOCATOR: Allocator = Allocator::new();

pub fn init() {
    unsafe { ALLOCATOR.init(HEAP_START as *mut u8, HEAP_SIZE) };
    crate::kprintln!(
        "Kernel heap initialized at {:#X}, of size {:#X}",
        HEAP_START,
        HEAP_SIZE
    )
}

struct Allocator {
    allocator: BlockAllocator,
}

impl Allocator {
    pub const fn new() -> Self {
        Self {
            allocator: BlockAllocator::new(),
        }
    }

    pub unsafe fn init(&self, heap_start: *mut u8, heap_size: u64) {
        self.allocator.lock().init(heap_start, heap_size)
    }
}

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        // critical section
        crate::interrupts::disable_interrupts();
        let ptr = self.allocator.alloc(layout);
        crate::interrupts::enable_interrupts();
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        crate::interrupts::disable_interrupts();
        self.allocator.dealloc(ptr, layout);
        crate::interrupts::enable_interrupts();
    }
}
