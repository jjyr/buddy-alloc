use crate::fast_alloc::{FastAlloc, FastAllocParam, BLOCK_SIZE};

#[repr(align(64))]
struct AlignedBuf([u8; 4096]);

impl Default for AlignedBuf {
    fn default() -> Self {
        Self([0u8; 4096])
    }
}

fn with_allocator<F: FnOnce(FastAlloc)>(f: F, buf: &[u8]) {
    let allocator = unsafe {
        let addr = buf.as_ptr();
        let len = buf.len();
        let param = FastAllocParam::new(addr, len);
        FastAlloc::new(param)
    };
    f(allocator);
}

#[test]
fn test_basic_malloc() {
    let buf = AlignedBuf::default();
    // alloc a min block
    with_allocator(
        |mut allocator| {
            let p = allocator.malloc(64);
            let p_addr = p as usize;
            assert!(!p.is_null());
            // memory writeable
            unsafe { p.write(42) };
            assert_eq!(p_addr, p as usize);
            assert_eq!(unsafe { *p }, 42);
        },
        &buf.0,
    );
}

#[test]
fn test_multiple_malloc() {
    let buf = AlignedBuf::default();
    with_allocator(
        |mut allocator| {
            let mut available_bytes = buf.0.len();
            // alloc serveral sized blocks
            while available_bytes >= BLOCK_SIZE {
                let bytes = BLOCK_SIZE;
                assert!(!allocator.malloc(bytes).is_null());
                available_bytes -= bytes;
            }
        },
        &buf.0,
    );
}

#[test]
fn test_small_size_malloc() {
    let buf = AlignedBuf::default();
    with_allocator(
        |mut allocator| {
            let mut available_bytes = buf.0.len();
            while available_bytes >= BLOCK_SIZE {
                assert!(!allocator.malloc(BLOCK_SIZE).is_null());
                available_bytes -= BLOCK_SIZE;
            }
            // memory should be drained, we can't allocate even 1 byte
            assert!(allocator.malloc(1).is_null());
        },
        &buf.0,
    );
}

#[test]
fn test_fail_malloc() {
    let buf = AlignedBuf::default();
    // not enough memory since we only have HEAP_SIZE bytes,
    // and the allocator itself occupied few bytes
    with_allocator(
        |mut allocator| {
            let p = allocator.malloc(BLOCK_SIZE + 1);
            assert!(p.is_null());
        },
        &buf.0,
    );
}

#[test]
fn test_malloc_and_free() {
    fn _test_malloc_and_free(times: usize) {
        let buf = AlignedBuf::default();
        with_allocator(
            |mut allocator| {
                for _i in 0..times {
                    let mut available_bytes = buf.0.len();
                    let mut ptrs = Vec::new();
                    // alloc serveral sized blocks
                    while available_bytes >= BLOCK_SIZE {
                        let bytes = BLOCK_SIZE;
                        let p = allocator.malloc(bytes);
                        assert!(!p.is_null());
                        ptrs.push(p);
                        available_bytes -= bytes;
                    }
                    // space is drained
                    assert!(allocator.malloc(1).is_null());
                    // free allocated blocks
                    for ptr in ptrs {
                        assert!(allocator.contains_ptr(ptr));
                        allocator.free(ptr);
                    }
                }
            },
            &buf.0,
        );
    }
    _test_malloc_and_free(10);
}

#[test]
fn test_free_bug() {
    let buf = AlignedBuf::default();
    with_allocator(
        |mut allocator| {
            let p1 = allocator.malloc(32);
            allocator.free(p1);
            let p2 = allocator.malloc(64);
            let p3 = allocator.malloc(61);
            allocator.free(p2);
            allocator.free(p3);
        },
        &buf.0,
    );
}
