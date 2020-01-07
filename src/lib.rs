#![cfg_attr(not(test), no_std)]

mod buddy_alloc;
mod non_threadsafe_alloc;
#[cfg(test)]
mod tests;

pub use crate::buddy_alloc::BuddyAlloc;
pub use non_threadsafe_alloc::NonThreadsafeAlloc;
