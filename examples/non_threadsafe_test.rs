use buddy_alloc::NonThreadsafeAlloc;

// 1M
const HEAP_SIZE: usize = 1024 * 1024;

pub static mut HEAP: [u8; HEAP_SIZE] = [0u8; HEAP_SIZE];

// This allocator can't work in tests since it's non-threadsafe.
#[cfg_attr(not(test), global_allocator)]
static ALLOC: NonThreadsafeAlloc = unsafe { NonThreadsafeAlloc::new(HEAP.as_ptr(), HEAP_SIZE) };

fn main() {
    let v = vec![0u8; 32];
    drop(v);
    let p1 = vec![0u8; 4096];
    let p2 = vec![0u8; 138];
    drop(p1);
    let msg = "alloc success".to_string();
    println!("{} {:?}", msg, p2.len());
}
