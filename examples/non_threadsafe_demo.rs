use buddy_alloc::{NonThreadsafeAlloc, REQUIRED_SPACE};
static mut HEAP: [u8; REQUIRED_SPACE] = [0u8; REQUIRED_SPACE];

// This allocator can't work in tests since it's non-threadsafe.
#[cfg_attr(not(test), global_allocator)]
static ALLOC: NonThreadsafeAlloc = unsafe { NonThreadsafeAlloc::new(HEAP.as_ptr()) };

fn main() {
    let v = vec![0u8; 42];
    let msg = "alloc success".to_string();
    println!("{} {:?}", msg, v.len());
}
