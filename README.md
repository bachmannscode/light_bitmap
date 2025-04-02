# Light Bitmap

[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/bachmannscode/light_bitmap/ci.yml?branch=main&logo=github" height="20">](https://github.com/bachmannscode/light_bitmap/actions)
[<img alt="crates.io" src="https://img.shields.io/crates/v/light_bitmap.svg" height="20">](https://crates.io/crates/light_bitmap)
[![dependency status](https://deps.rs/repo/github/bachmannscode/light_bitmap/status.svg)](https://deps.rs/repo/github/bachmannscode/light_bitmap)

<!-- cargo-rdme start -->

A minimal, fixed-size bitmap library written in pure Rust.  
`no_std`, no heap / `alloc`, no `unsafe` — just `core`.

Designed for use in embedded and resource-constrained environments.

[`BitMap`] is the main struct in this library. Its [features](#features)
are listed below.

## Examples
```rust
use light_bitmap::{bucket_count, BitMap};

const BIT_COUNT: usize = 10;
let mut bitmap = BitMap::<BIT_COUNT, { bucket_count(BIT_COUNT) }>::new();
bitmap.set(3);
assert!(bitmap.is_set(3));
assert_eq!(bitmap.popcount(), 1);
```

## Use Cases

- Embedded development
- Kernel / low-level systems
- Real-time control systems
- Bitmasking for constrained or deterministic environments

## Features

- `#![no_std]` compatible
- Bit-level operations on a fixed number of bits
- No heap allocations (stack-only)
- Const-generic API: `BitMap<const BIT_COUNT, const BUCKET_COUNT>`
- Efficient iteration over all, set or unset bits:
  - `iter()` (all bits as bools)
  - `iter_ones()` (indices of set bits)
  - `iter_zeros()` (indices of unset bits)
- Support for bitwise ops:
  - `&`, `|`, `^`, `!`
  - `<<`, `>>`
  - `&=`, `|=`, `^=`, `<<=`, `>>=`
- Range operations: `set_range`, `unset_range`
- Logical operations: `popcount`, `first_set_bit`
- Rotation support: `rotate_left`, `rotate_right`

<!-- cargo-rdme end -->

---

MIT licensed. Maintained with love for bitwise enthusiasts.
