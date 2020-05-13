# Buddy-alloc
[![Crates.io](https://img.shields.io/crates/v/buddy-alloc.svg)](https://crates.io/crates/buddy-alloc)


Buddy-alloc is a memory allocator for no-std Rust, used for embedded environments.

## Usage

Check [examples](https://github.com/jjyr/buddy-alloc/tree/master/examples) and [Rust Doc](https://docs.rs/buddy-alloc).

## Why

My original intention is to enable `alloc` crate for no-std Rust in CKB-VM without introducing LibC.
I choose the buddy allocation algorithm since it's simple, efficient enough, and It's easy to extended or composited with other memory allocation strategies.
This crate is designed to be used in general environment, it should be able to used in similar embedded environments.

## License

MIT
