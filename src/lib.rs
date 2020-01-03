//#![cfg_attr(not(test), no_std)]

mod buddy_alloc;
#[cfg(test)]
mod tests;

pub use crate::buddy_alloc::{BuddyAllocator, LEAF_SIZE, MAX_K};
