use crate::buddy_alloc::BuddyAlloc;
use core::alloc::{GlobalAlloc, Layout};
use core::cell::RefCell;

pub struct WrappedAlloc(RefCell<BuddyAlloc>);

unsafe impl GlobalAlloc for WrappedAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.0.borrow_mut().malloc(layout.size())
    }
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        self.0.borrow_mut().free(ptr);
    }
}
