# Buddy-alloc
[![Crates.io](https://img.shields.io/crates/v/buddy-alloc.svg)](https://crates.io/crates/buddy-alloc)


Buddy-alloc is a memory allocator for no-std Rust, used for embedded environments.

## Usage

Check [examples](https://github.com/jjyr/buddy-alloc/tree/master/examples) and [Rust Doc](https://docs.rs/buddy-alloc).

* This allocator is combined by a link-list based fast allocator and a buddy allocator.
* No syscalls, we assume the execution environment has no MMU, you need to pre-allocate the memory range for heaps.
* No threadsafe supports; you need to implement locks on your own.

## Why

My original intention is to enable `alloc` crate for no-std Rust in CKB-VM without introducing LibC.
I choose the buddy allocation algorithm since it's simple, stable, and efficient enough.
This crate is designed to be used in general environment, it should be able to used in similar embedded environments.

## License

MIT
