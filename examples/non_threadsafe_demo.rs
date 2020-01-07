use buddy_alloc::NonThreadsafeAlloc;

// 1M
const HEAP_SIZE: usize = 1024 * 1024;

static mut HEAP: [u8; HEAP_SIZE] = [0u8; HEAP_SIZE];

// a hack to get pointer address of HEAP in stable Rust
union Transmuter {
    from: [u8; HEAP_SIZE],
    to: usize,
}

// This allocator can't work in tests since it's non-threadsafe.
#[cfg_attr(not(test), global_allocator)]
static ALLOC: NonThreadsafeAlloc = unsafe {
    let addr = Transmuter { from: HEAP }.to;
    NonThreadsafeAlloc::new(addr, addr + HEAP_SIZE)
};

fn main() {
    let v = vec![0u8; 42];
    let msg = "alloc success".to_string();
    println!("{} {:?}", msg, v.len());
}
