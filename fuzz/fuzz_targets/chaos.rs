#![no_main]
use arbitrary::Arbitrary;
use buddy_alloc::{BuddyAllocParam, FastAllocParam, NonThreadsafeAlloc};
use libfuzzer_sys::fuzz_target;
use std::alloc::{GlobalAlloc, Layout};
use std::cmp::{max, min};

#[derive(Debug, Arbitrary)]
enum Action {
    // Allocate a chunk with the size specified.
    Alloc { size: u16, align_bit: u8 },
    // Free the pointer at the index specified.
    Free { index: u8 },
}

const FAST_HEAP_SIZE: usize = 32 * 1024; // 32 KB
const HEAP_SIZE: usize = 1024 * 1024; // 1M
const LEAF_SIZE: usize = 256;
#[repr(align(64))]
struct Heap<const S: usize>([u8; S]);
static mut FAST_HEAP: Heap<FAST_HEAP_SIZE> = Heap([0u8; FAST_HEAP_SIZE]);
static mut HEAP: Heap<HEAP_SIZE> = Heap([0u8; HEAP_SIZE]);

fuzz_target!(|data: (u16, u32, u8, Vec<Action>)| {
    let (fast_heap_size, heap_size, leaf_size, action_list) = data;
    let fast_heap_size = max(64, min(fast_heap_size as usize & 0xffc0, FAST_HEAP_SIZE));
    let heap_size = max(256, min(heap_size as usize & 0xffffffc0, HEAP_SIZE));
    let leaf_size = max(16, min(leaf_size as usize & 0xf0, LEAF_SIZE));

    #[allow(static_mut_refs)]
    let heap = unsafe {
        let fast_param = FastAllocParam::new(FAST_HEAP.0.as_ptr(), fast_heap_size);
        let buddy_param = BuddyAllocParam::new(HEAP.0.as_ptr(), heap_size, leaf_size);
        NonThreadsafeAlloc::new(fast_param, buddy_param)
    };
    let mut ptrs = Vec::<(*mut u8, Layout)>::new();

    for action in action_list {
        match action {
            Action::Alloc { size, align_bit } => {
                let layout = {
                    let align = 1_usize.rotate_left(align_bit as u32);
                    if align == 1 << 63 {
                        return;
                    }
                    Layout::from_size_align(size as usize, align).unwrap()
                };
                let ptr = unsafe { heap.alloc(layout) };
                if !ptr.is_null() {
                    ptrs.push((ptr, layout));
                }
            }
            Action::Free { index } => {
                if index as usize >= ptrs.len() {
                    return;
                }
                let (ptr, layout) = ptrs.swap_remove(index as usize);
                unsafe {
                    heap.dealloc(ptr, layout);
                }
            }
        }
    }

    // Free the remaining allocations
    for (ptr, layout) in ptrs {
        unsafe {
            heap.dealloc(ptr, layout);
        }
    }

    // Make sure we can allocate the full heap (no fragmentation)
    let full = Layout::from_size_align(heap_size, 1).unwrap();
    unsafe { heap.alloc(full) };
});
