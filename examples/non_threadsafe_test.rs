use buddy_alloc::{BuddyAllocParam, FastAllocParam, NonThreadsafeAlloc};

const FAST_HEAP_SIZE: usize = 32 * 1024; // 32 KB
const HEAP_SIZE: usize = 1024 * 1024; // 1M
const LEAF_SIZE: usize = 16;

#[repr(align(64))]
struct Heap<const S: usize>([u8; S]);

static mut FAST_HEAP: Heap<FAST_HEAP_SIZE> = Heap([0u8; FAST_HEAP_SIZE]);
static mut HEAP: Heap<HEAP_SIZE> = Heap([0u8; HEAP_SIZE]);

// This allocator can't work in tests since it's non-threadsafe.
#[cfg_attr(not(test), global_allocator)]
static ALLOC: NonThreadsafeAlloc = unsafe {
    let fast_param = FastAllocParam::new(FAST_HEAP.0.as_ptr(), FAST_HEAP_SIZE);
    let buddy_param = BuddyAllocParam::new(HEAP.0.as_ptr(), HEAP_SIZE, LEAF_SIZE);
    NonThreadsafeAlloc::new(fast_param, buddy_param)
};

#[allow(clippy::useless_vec)]
fn main() {
    let v = vec![0u8; 32];
    drop(v);
    let p1 = vec![0u8; 4096];
    let p2 = vec![0u8; 138];
    drop(p1);
    let msg = "alloc success".to_string();
    println!("{} {:?}", msg, p2.len());
}
