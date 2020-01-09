#![cfg_attr(not(test), no_std)]

pub mod buddy_alloc;
mod non_threadsafe_alloc;
#[cfg(test)]
mod tests;

pub use non_threadsafe_alloc::NonThreadsafeAlloc;
