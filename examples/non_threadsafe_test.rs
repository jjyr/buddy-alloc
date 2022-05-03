use buddy_alloc::{BuddyAllocParam, FreelistAllocParam, NonThreadsafeAlloc};

const FREELIST_HEAP_SIZE: usize = 32 * 1024; // 32 KB
const BUDDY_HEAP_SIZE: usize = 1024 * 1024; // 1M
const LEAF_SIZE: usize = 16;

pub static mut FAST_HEAP: [u8; FREELIST_HEAP_SIZE] = [0u8; FREELIST_HEAP_SIZE];
pub static mut HEAP: [u8; BUDDY_HEAP_SIZE] = [0u8; BUDDY_HEAP_SIZE];

// This allocator can't work in tests since it's non-threadsafe.
#[cfg_attr(not(test), global_allocator)]
static ALLOC: NonThreadsafeAlloc = unsafe {
    let freelist_param = FreelistAllocParam::new(FAST_HEAP.as_ptr(), FREELIST_HEAP_SIZE);
    let buddy_param = BuddyAllocParam::new(HEAP.as_ptr(), BUDDY_HEAP_SIZE, LEAF_SIZE);
    NonThreadsafeAlloc::new(freelist_param, buddy_param)
};

fn main() {
    let v = vec![0u8; 32];
    drop(v);
    let p1 = vec![0u8; 4096];
    let p2 = vec![0u8; 138];
    drop(p1);
    let msg = "alloc success".to_string();
    println!("{} {:?}", msg, p2.len());
}
