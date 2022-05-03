//! NonThreadSafeAlloc
//! An allocator that does not support thread-safe

use crate::buddy_alloc::{BuddyAlloc, BuddyAllocParam};
use crate::freelist_alloc::{FreelistAlloc, FreelistAllocParam, BLOCK_SIZE};
use core::alloc::{GlobalAlloc, Layout};
use core::cell::RefCell;

/// Use buddy allocator if request bytes is large than this,
/// otherwise use freelist allocator
const MAX_FREELIST_ALLOC_SIZE: usize = BLOCK_SIZE;

/// NonThreadsafeAlloc
/// perfect for single threaded devices
pub struct NonThreadsafeAlloc {
    freelist_alloc_param: FreelistAllocParam,
    inner_freelist_alloc: RefCell<Option<FreelistAlloc>>,
    buddy_alloc_param: BuddyAllocParam,
    inner_buddy_alloc: RefCell<Option<BuddyAlloc>>,
}

impl NonThreadsafeAlloc {
    /// see BuddyAlloc::new
    pub const fn new(
        freelist_alloc_param: FreelistAllocParam,
        buddy_alloc_param: BuddyAllocParam,
    ) -> Self {
        NonThreadsafeAlloc {
            inner_freelist_alloc: RefCell::new(None),
            inner_buddy_alloc: RefCell::new(None),
            freelist_alloc_param,
            buddy_alloc_param,
        }
    }

    unsafe fn fetch_freelist_alloc<R, F: FnOnce(&mut FreelistAlloc) -> R>(&self, f: F) -> R {
        let mut inner = self.inner_freelist_alloc.borrow_mut();
        if inner.is_none() {
            inner.replace(FreelistAlloc::new(self.freelist_alloc_param));
        }
        f(inner.as_mut().expect("nerver"))
    }

    unsafe fn fetch_buddy_alloc<R, F: FnOnce(&mut BuddyAlloc) -> R>(&self, f: F) -> R {
        let mut inner = self.inner_buddy_alloc.borrow_mut();
        if inner.is_none() {
            inner.replace(BuddyAlloc::new(self.buddy_alloc_param));
        }
        f(inner.as_mut().expect("nerver"))
    }
}

unsafe impl GlobalAlloc for NonThreadsafeAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let bytes = layout.size();
        // use BuddyAlloc if size is larger than MAX_freelist_ALLOC_SIZE
        if bytes > MAX_FREELIST_ALLOC_SIZE {
            self.fetch_buddy_alloc(|alloc| alloc.malloc(bytes))
        } else {
            // try freelist alloc, fallback to BuddyAlloc if failed
            let mut p = self.fetch_freelist_alloc(|alloc| alloc.malloc(bytes));
            if p.is_null() {
                p = self.fetch_buddy_alloc(|alloc| alloc.malloc(bytes));
            }
            p
        }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        let freed = self.fetch_freelist_alloc(|alloc| {
            if alloc.contains_ptr(ptr) {
                alloc.free(ptr);
                true
            } else {
                false
            }
        });
        if !freed {
            self.fetch_buddy_alloc(|alloc| alloc.free(ptr));
        }
    }
}

unsafe impl Sync for NonThreadsafeAlloc {}
