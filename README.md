# Buddy-alloc

Buddy-alloc is a Rust implemented allocator, used for embedded environments.

## Usage

Check [examples](https://github.com/jjyr/buddy-alloc/tree/master/examples) and [Rust Doc](https://docs.rs/buddy-alloc).

## Why

I want to use the `alloc` crate in the CKB-VM(an embedded-like environment) without introducing libc; to implement a pure Rust memory allocator comes to my head, buddy memory allocation is simple and efficient enough for my use case, it may be used in other similar embedded environments.

## License

MIT
