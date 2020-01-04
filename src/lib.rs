#![cfg_attr(not(test), no_std)]

mod buddy_alloc;
#[cfg(test)]
mod tests;

pub use crate::buddy_alloc::{
    first_down_k, first_up_k, BuddyAllocator, LEAF_SIZE, MAX_K, REQUIRED_SPACE,
};
