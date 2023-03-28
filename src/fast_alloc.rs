//! Fast allocator
//! Optimized for fixed small memory block.

// Fix size 64 Bytes
pub const BLOCK_SIZE: usize = 64;

// By default, initialize 4 nodes at most
pub const DEFAULT_INITIALIZED_NODES: usize = 4;

struct Node {
    next: *mut Node,
    prev: *mut Node,
}

impl Node {
    fn init(list: *mut Node) {
        unsafe {
            (*list).next = list;
            (*list).prev = list;
        }
    }

    fn remove(list: *mut Node) {
        unsafe {
            (*(*list).prev).next = (*list).next;
            (*(*list).next).prev = (*list).prev;
        }
    }

    fn pop(list: *mut Node) -> *mut Node {
        let n_list: *mut Node = unsafe { (*list).next };
        Self::remove(n_list);
        n_list
    }

    fn push(list: *mut Node, p: *mut u8) {
        let p = p.cast::<Node>();
        unsafe {
            let n_list: Node = Node {
                prev: list,
                next: (*list).next,
            };
            p.write_unaligned(n_list);
            (*(*list).next).prev = p;
            (*list).next = p;
        }
    }

    fn is_empty(list: *const Node) -> bool {
        unsafe { (*list).next as *const Node == list }
    }
}

#[derive(Clone, Copy)]
pub struct FastAllocParam {
    base_addr: *const u8,
    len: usize,
    initialized_nodes: usize,
}

impl FastAllocParam {
    pub const fn new(base_addr: *const u8, len: usize) -> Self {
        FastAllocParam {
            base_addr,
            len,
            initialized_nodes: DEFAULT_INITIALIZED_NODES,
        }
    }

    pub const fn new_with_initialized_nodes(
        base_addr: *const u8,
        len: usize,
        initialized_nodes: usize,
    ) -> Self {
        FastAllocParam {
            base_addr,
            len,
            initialized_nodes,
        }
    }
}

pub struct FastAlloc {
    /// memory start addr
    base_addr: usize,
    /// memory end addr
    end_addr: usize,
    /// next addr to allocate nodes
    next_addr: usize,
    free: *mut Node,
}

impl FastAlloc {
    /// # Safety
    ///
    /// The `base_addr..(base_addr + len)` must be allocated before using,
    /// and must guarantee no others write to the memory range, otherwise behavior is undefined.
    pub unsafe fn new(param: FastAllocParam) -> Self {
        let FastAllocParam {
            base_addr,
            len,
            initialized_nodes,
        } = param;
        let nblocks = len / BLOCK_SIZE;
        debug_assert_eq!(len % BLOCK_SIZE, 0);

        let base_addr = base_addr as usize;
        let end_addr = base_addr + nblocks * BLOCK_SIZE;

        // Actual blocks to create here
        let cblocks = core::cmp::min(nblocks, initialized_nodes);

        // initialize free list
        let free = base_addr as *mut Node;
        Node::init(free);

        let mut addr = base_addr;
        for _ in 1..cblocks {
            addr += BLOCK_SIZE;
            Node::push(free, addr as *mut u8);
        }

        FastAlloc {
            base_addr,
            end_addr,
            next_addr: addr + BLOCK_SIZE,
            free,
        }
    }

    pub fn contains_ptr(&self, p: *mut u8) -> bool {
        let addr = p as usize;
        addr >= self.base_addr && addr < self.end_addr
    }

    pub fn malloc(&mut self, nbytes: usize) -> *mut u8 {
        if nbytes > BLOCK_SIZE {
            return core::ptr::null_mut();
        }

        if self.free.is_null() {
            if self.next_addr < self.end_addr {
                let result = self.next_addr;
                self.next_addr += BLOCK_SIZE;
                return result as *mut u8;
            } else {
                return core::ptr::null_mut();
            }
        }

        let is_last = Node::is_empty(self.free);
        let p = Node::pop(self.free) as *mut u8;
        if is_last {
            self.free = core::ptr::null_mut();
        }
        p
    }

    pub fn free(&mut self, p: *mut u8) {
        debug_assert!(self.contains_ptr(p));
        if self.free.is_null() {
            let n = p.cast();
            Node::init(n);
            self.free = n;
        } else {
            Node::push(self.free, p);
        }
    }
}
