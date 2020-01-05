#![cfg_attr(not(test), no_std)]

pub mod buddy_alloc;
mod non_threadsafe_alloc;
#[cfg(test)]
mod tests;

pub use crate::buddy_alloc::{LEAF_SIZE, MAX_K, REQUIRED_SPACE};
pub use non_threadsafe_alloc::NonThreadsafeAlloc;
