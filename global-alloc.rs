use core::ptr::NonNull;
use palloc::{GlobalPalloc, SpinPalloc};

#[global_allocator]
static mut ALLOCATOR: SpinPalloc = SpinPalloc::empty();

unsafe {
    let heap_start = *(0x8000_0320 as *const u32);
    let mem_end = *(0x8000_0318 as *const u32) + 0x8000_0000;
    let heap_size = (mem_end - heap_start - 1024 * 1024) as usize; // Give the stack 1MB
    ALLOCATOR.init(NonNull::new(heap_start as *mut u8).unwrap(), heap_size);
}
