pub mod heap;

/// a complely unsafe memory copy, just like c's memcpy
pub unsafe fn mem_copy(dst: *mut u8, src: *const u8, len: usize) {
    for i in 0..len {
        dst.add(i).write_volatile(src.add(i).read_volatile())
    }
}
