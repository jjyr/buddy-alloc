//! Buddy alloc,
//! This code heavily references from https://pdos.csail.mit.edu/6.828/2019/lec/malloc.c
//! Check wiki to learn the algorithm: https://en.wikipedia.org/wiki/Buddy_memory_allocation
//!
//! The idea to to initialize base..end memory to leaf size, then merge them up.

const OOM_MSG: &str = "requires more memory space to initialize BuddyAlloc";
const LEAF_ALIGN_ERROR_MSG: &str = "leaf size must be align to 16 bytes";
/// required align to 16 bytes, since BuddyList takes 16 bytes on 64-bits machine.
pub const MIN_LEAF_SIZE_ALIGN: usize = 16;

pub const fn block_size(k: usize, leaf_size: usize) -> usize {
    (1 << k) * leaf_size
}

const fn nblock(k: usize, entries_size: usize) -> usize {
    1 << (entries_size - k - 1)
}

fn roundup(n: usize, sz: usize) -> usize {
    if n == 0 {
        return 0;
    }
    ((n - 1) / sz + 1) * sz
}

fn log2(mut n: usize) -> usize {
    let mut k = 0;
    while n > 1 {
        k += 1;
        n >>= 1;
    }
    k
}

fn bit_isset(bit_array: *const u8, i: usize) -> bool {
    unsafe {
        let b = bit_array.add(i / 8);
        let m = 1 << (i % 8);
        *b & m == m
    }
}

fn bit_set(bit_array: *mut u8, i: usize) {
    unsafe {
        let b = bit_array.add(i / 8);
        let m = 1 << (i % 8);
        *b |= m;
    }
}

fn bit_clear(bit_array: *mut u8, i: usize) {
    //debug_assert!(bit_isset(bit_array, i));
    unsafe {
        let b = bit_array.add(i / 8);
        let m = 1 << (i % 8);
        *b &= !m;
    }
}

/// Find a min k that great than n bytes
/// return (k, rounded_size)
pub fn first_up_k(n: usize, leaf_size: usize) -> (usize, usize) {
    let mut k = 0;
    let mut size = leaf_size;
    while size < n {
        k += 1;
        size *= 2;
    }
    (k, size)
}

struct BuddyList {
    next: *mut BuddyList,
    prev: *mut BuddyList,
}

impl BuddyList {
    fn init(list: *mut BuddyList) {
        unsafe {
            (*list).next = list;
            (*list).prev = list;
        }
    }

    fn remove(list: *mut BuddyList) {
        unsafe {
            debug_assert!(!(*list).prev.is_null());
            debug_assert!(!(*list).next.is_null());
            (*(*list).prev).next = (*list).next;
            (*(*list).next).prev = (*list).prev;
        }
    }

    fn pop(list: *mut BuddyList) -> *mut BuddyList {
        debug_assert!(!Self::is_empty(list));
        let n_list: *mut BuddyList = unsafe { (*list).next };
        Self::remove(n_list);
        n_list
    }

    fn push(list: *mut BuddyList, p: *mut u8) {
        let p = p.cast::<BuddyList>();
        unsafe {
            let n_list: BuddyList = BuddyList {
                prev: list,
                next: (*list).next,
            };
            p.write_unaligned(n_list);
            (*(*list).next).prev = p;
            (*list).next = p;
        }
    }

    fn is_empty(list: *const BuddyList) -> bool {
        unsafe { (*list).next as *const BuddyList == list }
    }
}

struct Entry {
    free: *mut BuddyList,
    /// Bit array to keep tracking alloc
    alloc: *mut u8,
    /// Bit array to keep tracking split
    split: *mut u8,
}

impl Default for Entry {
    fn default() -> Self {
        Entry {
            free: core::ptr::null_mut(),
            alloc: core::ptr::null_mut(),
            split: core::ptr::null_mut(),
        }
    }
}

pub struct BuddyAlloc {
    /// memory start addr
    base_addr: usize,
    /// memory end addr
    end_addr: usize,
    /// initialized memory addr
    initialized_addr: usize,
    entries: *mut Entry,
    entries_size: usize,
    /// min size of a block
    leaf_size: usize,
}

impl BuddyAlloc {
    /// # Safety
    ///
    /// The `base_addr..(base_addr + len)` must be allocated before using,
    /// and must guarantee no others write to the memory range, to avoid undefined behaviors.
    /// The new function panic if memory space not enough for initialize BuddyAlloc.
    pub unsafe fn new(base_addr: *const u8, len: usize, leaf_size: usize) -> Self {
        let mut base_addr = base_addr as usize;
        let end_addr = base_addr + len;
        assert!(
            leaf_size % MIN_LEAF_SIZE_ALIGN == 0 && leaf_size != 0,
            LEAF_ALIGN_ERROR_MSG
        );
        base_addr = roundup(base_addr, leaf_size);
        // we use (k + 1)-th entry's split flag to test existence of k-th entry's blocks;
        // to accoding this convention, we make a dummy (entries_size - 1)-th entry.
        // so we plus 2 on entries_size.
        let entries_size = log2(len / leaf_size) + 2;

        // alloc buddy allocator memory
        let used_bytes = core::mem::size_of::<Entry>() * entries_size;
        debug_assert!(len >= used_bytes, OOM_MSG);
        let entries = base_addr as *mut Entry;
        base_addr += used_bytes;

        let buddy_list_size = core::mem::size_of::<BuddyList>();
        // init entries free
        for k in 0..entries_size {
            // use one bit for per memory block
            debug_assert!(len >= buddy_list_size, OOM_MSG);
            let entry = entries.add(k).as_mut().expect("entry");
            entry.free = base_addr as *mut BuddyList;
            core::ptr::write_bytes(entry.free, 0, buddy_list_size);
            BuddyList::init(entry.free);
            base_addr += buddy_list_size;
        }

        // init alloc
        for k in 0..entries_size {
            // use one bit for per memory block
            let used_bytes = roundup(nblock(k, entries_size), 8) / 8;
            debug_assert!(len >= used_bytes, OOM_MSG);
            let entry = entries.add(k).as_mut().expect("entry");
            entry.alloc = base_addr as *mut u8;
            // mark all blocks as allocated
            core::ptr::write_bytes(entry.alloc, 255, used_bytes);
            base_addr += used_bytes;
        }

        // init split
        for k in 1..entries_size {
            // use one bit for per memory block
            let used_bytes = roundup(nblock(k, entries_size), 8) / 8;
            debug_assert!(len >= used_bytes, OOM_MSG);
            let entry = entries.add(k).as_mut().expect("entry");
            entry.split = base_addr as *mut u8;
            // mark all blocks as not splited
            core::ptr::write_bytes(entry.split, 0, used_bytes);
            base_addr += used_bytes;
        }

        assert!(end_addr >= base_addr, OOM_MSG);

        BuddyAlloc {
            base_addr,
            end_addr,
            initialized_addr: base_addr,
            entries,
            entries_size,
            leaf_size,
        }
    }

    /// return true if the size is able to initialize, otherwise return false
    fn can_lazy_init(&self, size: usize) -> bool {
        self.end_addr >= self.initialized_addr + size
    }

    /// Initialize size bytes and return the pointer of memory.
    /// This function provides a fast allocation, the allocated memory can be merged back in `free`
    /// function.
    ///
    /// size must be 2 ** k times of leaf_size, otherwise is undefined behaviour.
    fn lazy_init(&mut self, size: usize) -> *mut u8 {
        // since k is n times of leaf_size, we can find k for size
        let k = log2(size / self.leaf_size);
        debug_assert_eq!((1 << k) * self.leaf_size, size);

        // align initialized_addr to n times size
        let initialized_len = self.initialized_addr - self.base_addr;
        let rounded_len = roundup(initialized_len, size);

        let mut gap_size = rounded_len - initialized_len;
        let mut align_k = k;
        while gap_size > 0 {
            align_k -= 1;
            let block_size = (1 << align_k) * self.leaf_size;
            while block_size <= gap_size {
                gap_size -= block_size;
            }
        }

        self.initialized_addr = self.base_addr + rounded_len;

        // mark allocated memory block as alloced
        let entry = self.entry(k);
        let parent_entry = self.entry(k + 1);
        // mark block as allocated
        let block_index = self.block_index(k, self.initialized_addr as *const u8);
        let parent_block_index = self.block_index(k + 1, self.initialized_addr as *const u8);
        //dbg!("set alloc", k, block_index);
        bit_set(entry.alloc, block_index);
         ////dbg!("set parent alloc", k + 1, parent_block_index);
        // bit_set(parent_entry.alloc, parent_block_index);
        //if block_index % 2 != 0 {
            // mark parent's block as splited,
            bit_set(parent_entry.split, parent_block_index);
        //}

        let p = self.initialized_addr as *mut u8;
            //dbg!("lazy alloc", k + 1, parent_block_index, p, k, block_index);
        self.initialized_addr += size;
        p
    }

    pub fn malloc(&mut self, nbytes: usize) -> *mut u8 {
        let (fk, rounded_size) = first_up_k(nbytes, self.leaf_size);

        // always try lazy init first, since its effecient
        if self.can_lazy_init(rounded_size) {
            return self.lazy_init(rounded_size);
        }

        let mut k =
            match (fk..self.entries_size).find(|&k| !BuddyList::is_empty(self.entry(k).free)) {
                Some(k) => k,
                None => return core::ptr::null_mut(),
            };
        let p: *mut u8 = BuddyList::pop(self.entry(k).free) as *mut u8;
        bit_set(self.entry(k).alloc, self.block_index(k, p));
        // split memory
        // mark sub blocks as alloced & splited
        while k > fk {
            // buddy pointer
            let q: *mut u8 = (p as usize + block_size(k - 1, self.leaf_size)) as *mut u8;
            bit_set(self.entry(k).split, self.block_index(k, p));
            bit_set(self.entry(k - 1).alloc, self.block_index(k - 1, p));
            debug_assert!(!bit_isset(
                self.entry(k - 1).alloc,
                self.block_index(k - 1, q)
            ));
            BuddyList::push(self.entry(k - 1).free, q);
            k -= 1;
        }
        //dbg!("malloc", k, self.block_index(k, p));
        p
    }

    pub fn free(&mut self, mut p: *mut u8) {
        let mut k = self.block_k(p);
            //dbg!("free start", k, p );
        while k < (self.entries_size - 1) {
            let block_index = self.block_index(k, p);
            let entry = self.entry(k);
            //dbg!("free", k, block_index, p );
            bit_clear(entry.alloc, block_index);
            let buddy = if block_index % 2 == 0 {
                block_index + 1
            } else {
                block_index - 1
            };


            // calculate parent block index
            let q = self.block_ptr(k, buddy);

            // buddy is uninitialized
            //dbg!("check buddy initialized", k, buddy,q, unsafe{self.initialized_addr as *mut u8}, q as usize >= self.initialized_addr);
            if q as usize + block_size(k, self.leaf_size) > self.initialized_addr {
                break;
            }

            // merge with buddy if buddy is not alloced and parent is split
            if bit_isset(entry.alloc, buddy) {
                break;
            }

            let parent_block_index = if block_index % 2 == 0 {
                self.block_index(k + 1, p)
            } else {
                self.block_index(k + 1, q)
            };

            // if parent is not splited, which implies parent is not initialized, then stop merge
            let parent_entry = self.entry(k + 1);
            if !bit_isset(parent_entry.split, parent_block_index) {
                break;
            }

            //dbg!("remove", q, k, buddy, bit_isset(entry.alloc, buddy), k + 1, parent_block_index, (p, k, block_index));
            // we can safely merge buddy
            // remove buddy from free list
            BuddyList::remove(q.cast::<BuddyList>());
            // clear split bit
            bit_clear(parent_entry.split, parent_block_index);
            // update p pointer
            if buddy % 2 == 0 {
                p = q;
            }
            // move to next k
            k += 1;
        }
        debug_assert!(!bit_isset(self.entry(k).alloc, self.block_index(k, p)));
        BuddyList::push(self.entry(k).free, p);
    }

    /// available bytes
    pub fn available_bytes(&self) -> usize {
        (self.end_addr - self.base_addr) / self.leaf_size * self.leaf_size
    }

    fn entry(&self, i: usize) -> &Entry {
        if i >= self.entries_size {
            panic!("index out of range");
        }
        unsafe { self.entries.add(i).as_ref().expect("entry") }
    }

    /// find k of p
    fn block_k(&self, p: *const u8) -> usize {
        for k in 0..(self.entries_size - 1) {
            if bit_isset(self.entry(k + 1).split, self.block_index(k + 1, p)) {
                // debug_assert!(bit_isset(self.entry(k).alloc, self.block_index(k, p)));
                return k;
            }
            // if bit_isset(self.entry(k).alloc, self.block_index(k, p)) {
            //     return k;
            // }
        }
        0
    }

    /// block index of p under k
    fn block_index(&self, k: usize, p: *const u8) -> usize {
        if (p as usize) < self.base_addr {
            // TODO handle this outside
            panic!("out of memory");
        }
        let n = p as usize - self.base_addr;
        let index = n / block_size(k, self.leaf_size);
        debug_assert!(index < nblock(k, self.entries_size));
        index
    }

    /// block addr of index under k
    fn block_ptr(&self, k: usize, i: usize) -> *mut u8 {
        let n = i * block_size(k, self.leaf_size);
        (self.base_addr + n) as *mut u8
    }
}
