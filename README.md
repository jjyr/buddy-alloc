# Buddy-alloc

Buddy-alloc is a Rust implemented allocator, used for embedded environments.

## Why

I want to use the `alloc` crate in the CKB-VM(an embedded-like environment) without introducing LibC; so a pure Rust implementation allocator comes to my head, buddy memory allocation is simple and efficient enough for my use case, it may be used in other similar embedded environments.
