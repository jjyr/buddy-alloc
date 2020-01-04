/// the smallest allocation bytes
pub const LEAF_SIZE: usize = 16;
/// max leaf index, implies that the max capable is to alloc 1MB memory at a time
pub const MAX_K: usize = 16;
/// min alloc space
pub const MIN_ALLOC_SPACE: usize = 1000_000;

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
        *b = *b & !m;
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
    core::cmp::min(k, MAX_K)
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

#[derive(Debug)]
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
        unsafe { (*(*list).prev).next = (*list).next };
    }

    fn pop(list: *mut BuddyList) -> *mut BuddyList {
        assert!(!Self::is_empty(list));
        let n_list: *mut BuddyList = unsafe { (*list).next };
        Self::remove(n_list);
        return n_list;
    }

    fn push(list: *mut BuddyList, p: *const u8) {
        let n_list: *mut BuddyList = p as *mut BuddyList;
        unsafe {
            (*n_list).next = (*list).next;
            (*n_list).prev = list;
            (*(*list).next).prev = n_list;
            (*list).next = n_list;
        }
    }

    fn is_empty(list: *mut BuddyList) -> bool {
        unsafe { (*list).next == list }
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

pub struct BuddyAllocator {
    /// which free list start from
    base_addr: usize,
    /// lower_addr after allocated
    lower_addr: usize,
    /// higher_addr
    higher_addr: usize,
    entries: [Entry; MAX_K + 1],
}

impl BuddyAllocator {
    pub fn new(mut lower_addr: usize, higher_addr: usize) -> Self {
        if higher_addr < lower_addr + MIN_ALLOC_SPACE {
            panic!(
                "alloc space is not enough, buddy allocator need at least {} bytes",
                MIN_ALLOC_SPACE
            );
        }
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

        let base_addr = lower_addr;
        // fill free list
        while higher_addr.saturating_sub(lower_addr) > LEAF_SIZE {
            let k = match first_down_k(higher_addr - lower_addr) {
                Some(k) => k,
                None => break,
            };
            BuddyList::push(entries[k].free, lower_addr as *const u8);
            lower_addr += block_size(k);
        }
        assert!(
            lower_addr < higher_addr,
            "Alloc space is not enough: lower_addr {}, higher_addr {}",
            lower_addr,
            higher_addr
        );
        BuddyAllocator {
            base_addr,
            lower_addr,
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
            let q: *const u8 = (p as usize + block_size(k - 1)) as *const u8;
            bit_set(self.entries[k].split, self.block_index(k, p));
            bit_set(self.entries[k - 1].alloc, self.block_index(k - 1, p));
            BuddyList::push(self.entries[k - 1].free, q);
            k -= 1;
        }
        p
    }

    /// total available bytes
    pub fn available_bytes(&self) -> usize {
        self.lower_addr - self.base_addr
    }

    /// wasted bytes due to buddy algorithm
    pub fn wasted_bytes(&self) -> usize {
        self.higher_addr - self.lower_addr
    }

    fn block_index(&self, k: usize, p: *const u8) -> usize {
        let n = p as usize - self.base_addr;
        n / block_size(k)
    }
}
