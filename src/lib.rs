#![cfg_attr(not(test), no_std)]

pub mod buddy_alloc;
#[cfg(test)]
mod tests;
mod wrapped_alloc;

pub use crate::buddy_alloc::{LEAF_SIZE, MAX_K, REQUIRED_SPACE};
pub use wrapped_alloc::WrappedAlloc as Allocator;
