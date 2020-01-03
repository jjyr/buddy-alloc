use crate::{BuddyAllocator, LEAF_SIZE, MAX_K};
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
fn test_fail_malloc() {
    // not enough memory
    // since we only have 1024 bytes, and the allocator itself occupied few bytes
    let p = new_allocator().malloc(MALLOC_SIZE);
    assert!(p.is_null());
}
