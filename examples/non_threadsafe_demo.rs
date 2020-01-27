use buddy_alloc::NonThreadsafeAlloc;

// 1M
const HEAP_SIZE: usize = 1024 * 1024;
const LEAF_SIZE: usize = 16;

pub static mut HEAP: [u8; HEAP_SIZE] = [0u8; HEAP_SIZE];

// This allocator can't work in tests since it's non-threadsafe.
#[cfg_attr(not(test), global_allocator)]
static ALLOC: NonThreadsafeAlloc =
    unsafe { NonThreadsafeAlloc::new(HEAP.as_ptr(), HEAP_SIZE, LEAF_SIZE) };

fn main() {
    let v = vec![0u8; 42];
    let msg = "alloc success".to_string();
    println!("{} {:?}", msg, v.len());
}
