#![cfg_attr(not(test), no_std)]

pub mod buddy_alloc;
pub mod fast_alloc;
pub mod non_threadsafe_alloc;
#[cfg(test)]
mod tests;

pub use crate::buddy_alloc::BuddyAllocParam;
pub use crate::fast_alloc::FastAllocParam;
pub use crate::non_threadsafe_alloc::NonThreadsafeAlloc;
