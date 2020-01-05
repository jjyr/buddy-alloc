//! Buddy alloc,
//! This code heavily references from https://pdos.csail.mit.edu/6.828/2019/lec/malloc.c
//! Check wiki to learn the algorithm: https://en.wikipedia.org/wiki/Buddy_memory_allocation
//!
//! For simplify, we only implement the fixed memory for now,
//! which means the total memory never grows once the allocator is created.
//! The user must initialize allocator with REQUIRED_SPACE addr ranges.

#![allow(clippy::needless_range_loop)]

/// the smallest allocation bytes
pub const LEAF_SIZE: usize = 16;
/// max leaf index, implies that the max capable is to alloc 1MB memory at a time
pub const MAX_K: usize = 16;
/// min alloc space
pub const REQUIRED_SPACE: usize = 1_073_428;

pub const fn block_size(k: usize) -> usize {
    (1 << k) * LEAF_SIZE
}

const fn nblock(k: usize) -> usize {
    1 << (MAX_K - k)
}

const fn roundup(n: usize, sz: usize) -> usize {
    ((n - 1) / sz + 1) * sz
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
    unsafe {
        let b = bit_array.add(i / 8);
        let m = 1 << (i % 8);
        *b &= !m;
    }
}

// find a min k that great than n bytes
pub fn first_up_k(n: usize) -> usize {
    let mut k = 0;
    let mut size = LEAF_SIZE;
    while size < n {
        k += 1;
        size *= 2;
    }
    k
}

// find a max k that less than n bytes
pub fn first_down_k(n: usize) -> Option<usize> {
    let mut k: usize = 0;
    let mut size = LEAF_SIZE;
    while size < n {
        k += 1;
        size *= 2;
    }
    let k = if size != n { k.checked_sub(1) } else { Some(k) };
    k.map(|k| core::cmp::min(k, MAX_K))
}

struct BuddyList {
    next: *mut BuddyList,
    prev: *mut BuddyList,
}

impl core::fmt::Debug for BuddyList {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "BuddyList {{ ")?;
        if Self::is_empty(self as *const BuddyList) {
            write!(f, "empty")?;
        } else {
            let mut count = 0;
            let mut p = self as *const BuddyList;
            while count == 0 || p != self as *const BuddyList {
                unsafe {
                    write!(
                        f,
                        "item({}, self: {:?}, next: {:?}, prev: {:?}) ",
                        count,
                        p,
                        (*p).next,
                        (*p).prev
                    )?;
                }
                count += 1;
                p = unsafe { (*p).next };

                if count > 10 {
                    write!(f, "items...")?;
                    break;
                }
            }
        }
        write!(f, " }}")
    }
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
            (*(*list).prev).next = (*list).next;
            (*(*list).next).prev = (*list).prev;
        }
    }

    fn pop(list: *mut BuddyList) -> *mut BuddyList {
        assert!(!Self::is_empty(list));
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
    /// space start addr
    base_addr: usize,
    /// space end addr
    higher_addr: usize,
    entries: [Entry; MAX_K + 1],
}

impl BuddyAlloc {
    pub fn required_space() -> usize {
        let mut space = 0;
        // allocator cost
        for k in 0..=MAX_K {
            space += core::mem::size_of::<BuddyList>();
            let used_bytes = roundup(nblock(k), 8) / 8;
            space += used_bytes
        }
        for k in 1..=MAX_K {
            let used_bytes = roundup(nblock(k), 8) / 8;
            space += used_bytes
        }
        // usage space
        space + block_size(MAX_K)
    }

    pub fn new(mut lower_addr: usize) -> Self {
        let higher_addr = lower_addr + REQUIRED_SPACE;

        // alloc buddy allocator memory
        let mut entries: [Entry; MAX_K + 1] = Default::default();

        // init entires free list
        for k in 0..=MAX_K {
            // use one bit for per memory block
            let used_bytes = core::mem::size_of::<BuddyList>();
            entries[k].free = lower_addr as *mut BuddyList;
            BuddyList::init(entries[k].free);
            lower_addr += used_bytes;
        }

        // init alloc
        for k in 0..=MAX_K {
            // use one bit for per memory block
            let used_bytes = roundup(nblock(k), 8) / 8;
            entries[k].alloc = lower_addr as *mut u8;
            lower_addr += used_bytes;
        }

        // init split
        for k in 1..=MAX_K {
            // use one bit for per memory block
            let used_bytes = roundup(nblock(k), 8) / 8;
            entries[k].split = lower_addr as *mut u8;
            lower_addr += used_bytes;
        }

        assert_eq!(
            higher_addr - lower_addr,
            block_size(MAX_K),
            "not satisfied required space"
        );
        BuddyList::push(entries[MAX_K].free, lower_addr as *mut u8);
        BuddyAlloc {
            base_addr: lower_addr,
            higher_addr,
            entries,
        }
    }

    pub fn malloc(&mut self, nbytes: usize) -> *mut u8 {
        let fk = first_up_k(nbytes);
        let mut k = match (fk..=MAX_K).find(|&k| !BuddyList::is_empty(self.entries[k].free)) {
            Some(k) => k,
            None => return core::ptr::null_mut(),
        };
        let p: *mut u8 = BuddyList::pop(self.entries[k].free) as *mut u8;
        bit_set(self.entries[k].alloc, self.block_index(k, p));
        while k > fk {
            let q: *mut u8 = (p as usize + block_size(k - 1)) as *mut u8;
            bit_set(self.entries[k].split, self.block_index(k, p));
            bit_set(self.entries[k - 1].alloc, self.block_index(k - 1, p));
            BuddyList::push(self.entries[k - 1].free, q);
            k -= 1;
        }
        p
    }

    pub fn free(&mut self, mut p: *mut u8) {
        let mut k = self.block_k(p);
        while k < MAX_K {
            let block_index = self.block_index(k, p);
            bit_clear(self.entries[k].alloc, block_index);
            let buddy = if block_index % 2 == 0 {
                block_index + 1
            } else {
                block_index - 1
            };
            if bit_isset(self.entries[k].alloc, buddy) {
                break;
            }
            // merge buddy since its free
            // 1. clear split of k + 1
            // 2. set p to the address of merged block
            // 3. repeat for k = k + 1 until reach MAX_K
            // 4. push p back to k entry free list
            let q = self.block_addr(k, buddy);
            BuddyList::remove(q as *mut BuddyList);
            if buddy % 2 == 0 {
                p = q as *mut u8;
            }
            bit_clear(self.entries[k + 1].split, self.block_index(k + 1, p));
            k += 1;
        }
        BuddyList::push(self.entries[k].free, p);
    }

    /// available bytes
    pub fn available_bytes(&self) -> usize {
        self.higher_addr - self.base_addr
    }

    /// wasted bytes due to buddy algorithm
    pub fn wasted_bytes(&self) -> usize {
        REQUIRED_SPACE - self.available_bytes()
    }

    /// find k of p
    fn block_k(&self, p: *const u8) -> usize {
        for k in 0..MAX_K {
            if bit_isset(self.entries[k + 1].split, self.block_index(k + 1, p)) {
                return k;
            }
        }
        0
    }

    /// block index of p under k
    fn block_index(&self, k: usize, p: *const u8) -> usize {
        let n = p as usize - self.base_addr;
        n / block_size(k)
    }

    /// block addr of index under k
    fn block_addr(&self, k: usize, i: usize) -> usize {
        let n = i * block_size(k);
        self.base_addr + n
    }
}
