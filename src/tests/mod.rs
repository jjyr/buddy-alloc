use crate::{
    buddy_alloc::{block_size, first_down_k},
    BuddyAllocator, LEAF_SIZE, REQUIRED_SPACE,
};

fn with_allocator<F: FnOnce(BuddyAllocator)>(f: F) {
    use std::alloc::{alloc, dealloc, Layout};
    let layout = Layout::new::<[u8; REQUIRED_SPACE]>();
    unsafe {
        let mem = alloc(layout);
        let lower_addr = mem as usize;
        let higher_addr = mem.add(REQUIRED_SPACE) as usize;
        let allocator = BuddyAllocator::new(lower_addr, higher_addr);
        f(allocator);
        dealloc(mem, layout);
    }
}

#[test]
fn test_required_space() {
    assert_eq!(BuddyAllocator::required_space(), REQUIRED_SPACE);
}

#[test]
fn test_basic_malloc() {
    // alloc a min block
    with_allocator(|mut allocator| {
        let p = allocator.malloc(512);
        let p_addr = p as usize;
        assert!(!p.is_null());
        // memory writeable
        unsafe { p.write(42) };
        assert_eq!(p_addr, p as usize);
        assert_eq!(unsafe { *p }, 42);
    });
}

#[test]
fn test_multi_size_malloc() {
    with_allocator(|mut allocator| {
        let mut available_bytes = allocator.available_bytes();
        let mut count = 0;
        // alloc serveral sized blocks
        while available_bytes >= LEAF_SIZE {
            let k = first_down_k(available_bytes - 1).unwrap_or_default();
            let bytes = block_size(k);
            assert!(!allocator.malloc(bytes).is_null());
            available_bytes -= bytes;
            count += 1;
        }
        assert_eq!(count, 17);
    });
}

#[test]
fn test_small_size_malloc() {
    with_allocator(|mut allocator| {
        let mut available_bytes = allocator.available_bytes();
        while available_bytes >= LEAF_SIZE {
            assert!(!allocator.malloc(LEAF_SIZE).is_null());
            available_bytes -= LEAF_SIZE;
        }
        // memory should be drained, we can't allocate even 1 byte
        assert!(allocator.malloc(1).is_null());
    });
}

#[test]
fn test_fail_malloc() {
    // not enough memory since we only have MALLOC_SIZE bytes,
    // and the allocator itself occupied few bytes
    with_allocator(|mut allocator| {
        let p = allocator.malloc(REQUIRED_SPACE);
        assert!(p.is_null());
    });
}

#[test]
fn test_malloc_and_free() {
    with_allocator(|mut allocator| {
        for _i in 0..10 {
            let mut available_bytes = allocator.available_bytes();
            let mut ptrs = Vec::new();
            // alloc serveral sized blocks
            while available_bytes >= LEAF_SIZE {
                let k = first_down_k(available_bytes - 1).unwrap_or_default();
                let bytes = block_size(k);
                let p = allocator.malloc(bytes);
                assert!(!p.is_null());
                ptrs.push(p);
                available_bytes -= bytes;
            }
            // space is drained
            assert!(allocator.malloc(1).is_null());
            // free allocated blocks
            for ptr in ptrs {
                allocator.free(ptr);
            }
        }
    });
}
