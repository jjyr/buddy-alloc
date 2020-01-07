use crate::buddy_alloc::BuddyAlloc;
use core::alloc::{GlobalAlloc, Layout};
use core::cell::RefCell;

/// NonThreadsafeAlloc
/// perfect for single threaded devices
pub struct NonThreadsafeAlloc {
    inner: RefCell<Option<BuddyAlloc>>,
    base_addr: usize,
    end_addr: usize,
}

impl NonThreadsafeAlloc {
    /// see BuddyAlloc::new
    pub const fn new(base_addr: usize, end_addr: usize) -> Self {
        NonThreadsafeAlloc {
            inner: RefCell::new(None),
            base_addr,
            end_addr,
        }
    }

    unsafe fn fetch_inner<R, F: FnOnce(&mut BuddyAlloc) -> R>(&self, f: F) -> R {
        if self.inner.borrow().is_none() {
            self.inner
                .borrow_mut()
                .replace(BuddyAlloc::new(self.base_addr, self.end_addr));
        }
        let mut inner = self.inner.borrow_mut();
        f(inner.as_mut().expect("nerver"))
    }
}

unsafe impl GlobalAlloc for NonThreadsafeAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.fetch_inner(|alloc| alloc.malloc(layout.size()))
    }
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        self.fetch_inner(|alloc| alloc.free(ptr));
    }
}

unsafe impl Sync for NonThreadsafeAlloc {}
