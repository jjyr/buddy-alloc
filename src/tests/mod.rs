use crate::{
    buddy_alloc::{block_size, first_down_k},
    BuddyAllocator, LEAF_SIZE, MAX_K,
};
use std::alloc::{alloc, Layout};

const MALLOC_SIZE: usize = 1000_000;

fn new_allocator() -> BuddyAllocator {
    let layout = Layout::new::<[u8; MALLOC_SIZE]>();
    unsafe {
        let mem = alloc(layout);
        let lower_addr = mem as usize;
        let higher_addr = mem.add(MALLOC_SIZE) as usize;
        BuddyAllocator::new(lower_addr, higher_addr)
    }
}

#[test]
fn test_basic_malloc() {
    // alloc a min block
    let mut allocator = new_allocator();
    let p = allocator.malloc(512);
    let p_addr = p as usize;
    assert!(!p.is_null());
    // memory writeable
    unsafe { p.write(42) };
    assert_eq!(p_addr, p as usize);
    assert_eq!(unsafe { *p }, 42);
}

#[test]
fn test_multi_size_malloc() {
    let mut allocator = new_allocator();
    let mut available_bytes = allocator.available_bytes();
    let mut count = 0;
    // alloc serveral sized blocks
    while available_bytes >= LEAF_SIZE {
        let k = first_down_k(available_bytes).unwrap();
        let bytes = block_size(k);
        assert!(!allocator.malloc(bytes).is_null());
        available_bytes -= bytes;
        count += 1;
    }
    assert_eq!(count, 8);
}

#[test]
fn test_small_size_malloc() {
    let mut allocator = new_allocator();
    let mut available_bytes = allocator.available_bytes();
    while available_bytes >= LEAF_SIZE {
        assert!(!allocator.malloc(LEAF_SIZE).is_null());
        available_bytes -= LEAF_SIZE;
    }
    // memory should be drained, we can't allocate even 1 byte
    assert!(allocator.malloc(1).is_null());
}

#[test]
fn test_fail_malloc() {
    // not enough memory
    // since we only have 1024 bytes, and the allocator itself occupied few bytes
    let p = new_allocator().malloc(MALLOC_SIZE);
    assert!(p.is_null());
}
