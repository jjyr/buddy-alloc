use crate::buddy_alloc::{block_size, BuddyAlloc, BuddyAllocParam, MIN_LEAF_SIZE_ALIGN};

const HEAP_SIZE: usize = 1024 * 1024;
const LEAF_SIZE: usize = MIN_LEAF_SIZE_ALIGN;

fn with_allocator<F: Fn(BuddyAlloc)>(heap_size: usize, leaf_size: usize, f: F) {
    let buf: Vec<u8> = Vec::with_capacity(heap_size);
    let param = BuddyAllocParam::new(buf.as_ptr(), heap_size, leaf_size);
    unsafe {
        let allocator = BuddyAlloc::new(param);
        f(allocator);
    }

    let zero_filled_buf: Vec<u8> = vec![0; heap_size];
    let param =
        BuddyAllocParam::new_with_zero_filled(zero_filled_buf.as_ptr(), heap_size, leaf_size);
    unsafe {
        let allocator = BuddyAlloc::new(param);
        f(allocator);
    }
}

// find a max k that less than n bytes
pub fn first_down_k(n: usize) -> Option<usize> {
    let mut k: usize = 0;
    let mut size = LEAF_SIZE;
    while size < n {
        k += 1;
        size *= 2;
    }
    if size != n {
        k.checked_sub(1)
    } else {
        Some(k)
    }
}

#[test]
fn test_available_bytes() {
    with_allocator(HEAP_SIZE, LEAF_SIZE, |allocator| {
        let available_bytes = allocator.available_bytes();
        assert!(available_bytes > (HEAP_SIZE as f64 * 0.8) as usize);
    });
}

#[test]
fn test_basic_malloc() {
    // alloc a min block
    with_allocator(HEAP_SIZE, LEAF_SIZE, |mut allocator| {
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
fn test_multiple_malloc() {
    with_allocator(HEAP_SIZE, LEAF_SIZE, |mut allocator| {
        let mut available_bytes = allocator.available_bytes();
        let mut count = 0;
        // alloc serveral sized blocks
        while available_bytes >= LEAF_SIZE {
            let k = first_down_k(available_bytes - 1).unwrap_or_default();
            let bytes = block_size(k, LEAF_SIZE);
            assert!(!allocator.malloc(bytes).is_null());
            available_bytes -= bytes;
            count += 1;
        }
        assert_eq!(count, 11);
    });
}

#[test]
fn test_small_size_malloc() {
    with_allocator(HEAP_SIZE, LEAF_SIZE, |mut allocator| {
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
    // not enough memory since we only have HEAP_SIZE bytes,
    // and the allocator itself occupied few bytes
    with_allocator(HEAP_SIZE, LEAF_SIZE, |mut allocator| {
        let p = allocator.malloc(HEAP_SIZE);
        assert!(p.is_null());
    });
}

#[test]
fn test_malloc_and_free() {
    fn _test_malloc_and_free(times: usize, heap_size: usize) {
        with_allocator(heap_size, LEAF_SIZE, |mut allocator| {
            for _i in 0..times {
                let mut available_bytes = allocator.available_bytes();
                let mut ptrs = Vec::new();
                // alloc serveral sized blocks
                while available_bytes >= LEAF_SIZE {
                    let k = first_down_k(available_bytes - 1).unwrap_or_default();
                    let bytes = block_size(k, LEAF_SIZE);
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
    // test with heaps: 1M, 2M, 4M, 8M
    for i in &[1, 2, 4, 8] {
        _test_malloc_and_free(10, i * HEAP_SIZE);
    }
}

#[test]
fn test_free_bug() {
    with_allocator(HEAP_SIZE, LEAF_SIZE, |mut allocator| {
        let p1 = allocator.malloc(32);
        allocator.free(p1);
        let p2 = allocator.malloc(4096);
        let p3 = allocator.malloc(138);
        allocator.free(p2);
        allocator.free(p3);
    });
}

#[test]
fn test_malloc_and_free_gap() {
    // malloc 1 k and 2 k alternately, then consumes remain memory
    fn _test_malloc_and_free_gap(times: usize, heap_size: usize, leaf_size: usize) {
        with_allocator(heap_size, leaf_size, |mut allocator| {
            let blocks_num = allocator.available_bytes() / leaf_size;

            for _i in 0..times {
                let mut available_bytes = allocator.available_bytes();
                let mut ptrs = Vec::new();
                // align blocks to n times of 4
                for _j in 0..blocks_num / 4 {
                    // alloc 1 k block
                    let bytes = block_size(1, leaf_size) >> 1;
                    let p = allocator.malloc(bytes);
                    assert!(!p.is_null());
                    ptrs.push(p);
                    available_bytes -= bytes;
                    // alloc 2 k block
                    let bytes = block_size(2, leaf_size) >> 1;
                    let p = allocator.malloc(bytes);
                    assert!(!p.is_null());
                    ptrs.push(p);
                    available_bytes -= bytes;
                }

                for _j in 0..blocks_num / 4 {
                    // alloc 1 k block
                    let bytes = block_size(1, leaf_size) >> 1;
                    let p = allocator.malloc(bytes);
                    assert!(!p.is_null());
                    ptrs.push(p);
                    available_bytes -= bytes;
                }
                // calculate remain blocks
                let remain_blocks = blocks_num - blocks_num / 4 * 4;
                assert_eq!(available_bytes, remain_blocks * leaf_size);
                // space is drained
                for _ in 0..remain_blocks {
                    let p = allocator.malloc(leaf_size);
                    assert!(!p.is_null());
                    ptrs.push(p);
                }
                assert!(allocator.malloc(1).is_null());
                // free allocated blocks
                for ptr in ptrs {
                    allocator.free(ptr);
                }
            }
        });
    }

    // test with heaps: 1M, 2M, 4M, 8M
    for i in &[1, 2, 4, 8] {
        _test_malloc_and_free_gap(10, i * HEAP_SIZE, LEAF_SIZE);
    }
}

#[test]
fn test_example_bug() {
    // simulate example bug
    with_allocator(HEAP_SIZE, LEAF_SIZE, |mut allocator| {
        let mut ptrs = Vec::new();
        ptrs.push(allocator.malloc(4));
        ptrs.push(allocator.malloc(5));
        allocator.free(ptrs[0]);
        ptrs.push(allocator.malloc(40));
        ptrs.push(allocator.malloc(48));
        ptrs.push(allocator.malloc(80));
        ptrs.push(allocator.malloc(42));
        ptrs.push(allocator.malloc(13));
        ptrs.push(allocator.malloc(8));
        ptrs.push(allocator.malloc(24));
        ptrs.push(allocator.malloc(16));
        ptrs.push(allocator.malloc(1024));
        ptrs.push(allocator.malloc(104));
        ptrs.push(allocator.malloc(8));
        for ptr in ptrs.into_iter().skip(1) {
            allocator.free(ptr);
        }
    });
}

#[test]
fn test_alignment() {
    let data = [0u8; 4 << 16];
    println!("Buffer data: {:p}", data.as_ptr());
    let mut allocator =
        unsafe { BuddyAlloc::new(BuddyAllocParam::new(data.as_ptr(), 4 << 16, 4096)) };
    let p = allocator.malloc(4);
    println!("Allocated pointer: {:p}", p);
}
