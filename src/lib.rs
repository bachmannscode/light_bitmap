#![cfg_attr(not(feature = "std"), no_std)]

//! A minimal, fixed-size bitmap library written in pure Rust.  
//! `no_std`, no heap / `alloc`, no `unsafe` — just `core`.
//!
//! Designed for use in embedded and resource-constrained environments.
//!
//! [`BitMap`] is the main struct in this library. Its [features](#features)
//! are listed below.
//!
//! # Examples
//! ```
//! use light_bitmap::{bucket_count, BitMap};
//!
//! const BIT_COUNT: usize = 10;
//! let mut bitmap = BitMap::<BIT_COUNT, { bucket_count(BIT_COUNT) }>::new();
//! bitmap.set(3);
//! assert!(bitmap.is_set(3));
//! assert_eq!(bitmap.popcount(), 1);
//! ```
//!
//! # Use Cases
//!
//! - Embedded development
//! - Kernel / low-level systems
//! - Real-time control systems
//! - Bitmasking for constrained or deterministic environments
//!
//! # Features
//!
//! - `#![no_std]` compatible
//! - Bit-level operations on a fixed number of bits
//! - No heap allocations (stack-only)
//! - Const-generic API: `BitMap<const BIT_COUNT, const BUCKET_COUNT>`
//! - Efficient iteration over all, set or unset bits:
//!   - `iter()` (all bits as bools)
//!   - `iter_ones()` (indices of set bits)
//!   - `iter_zeros()` (indices of unset bits)
//! - Support for bitwise ops:
//!   - `&`, `|`, `^`, `!`
//!   - `<<`, `>>`
//!   - `&=`, `|=`, `^=`, `<<=`, `>>=`
//! - Range operations: `set_range`, `unset_range`
//! - Logical operations: `popcount`, `first_set_bit`
//! - Rotation support: `rotate_left`, `rotate_right`

use core::array::from_fn;
use core::fmt::{Debug, Formatter};
use core::iter::{FusedIterator, Iterator};
use core::ops::{
    BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Range, Shl, ShlAssign,
    Shr, ShrAssign,
};

#[cfg(all(test, feature = "std"))]
mod tests;

/// Computes the number of buckets needed to store `bit_count` bits.
///
/// It's recommended to inline this call as a const expression into the type
/// annotation generics to avoid unnecessary panics.
///
/// # Examples
/// ```
/// use light_bitmap::bucket_count;
///
/// assert_eq!(bucket_count(9), 2);
/// assert_eq!(bucket_count(16), 2);
/// assert_eq!(bucket_count(17), 3);
/// ```
pub const fn bucket_count(bit_count: usize) -> usize {
    bit_count.div_ceil(8)
}

#[allow(clippy::no_effect)]
const fn assert_nonzero_bitcount(bit_count: usize) {
    // This will cause a compile-time error if bit_count == 0
    ["BIT_COUNT must be greater than zero"][(bit_count == 0) as usize];
}

/// The main type that stores the information.
///
/// `BIT_COUNT` is the number of usable bits.
/// `BUCKET_COUNT` is the number of internal buckets needed and should only be
/// set via const expression with [`bucket_count`] to avoid unnecessary panics
/// (see [`new`]).
///
/// Internally stores bits in an array of `u8`.
///
/// [`new`]: BitMap::new
#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct BitMap<const BIT_COUNT: usize, const BUCKET_COUNT: usize>([u8; BUCKET_COUNT]);

impl<const BIT_COUNT: usize, const BUCKET_COUNT: usize> BitMap<BIT_COUNT, BUCKET_COUNT> {
    /// Creates a new bitmap with all bits set to `false`.
    ///
    /// # Examples
    /// ```
    /// use light_bitmap::{bucket_count, BitMap};
    ///
    /// let bitmap = BitMap::<16, { bucket_count(16) }>::new();
    /// assert_eq!(bitmap.popcount(), 0);
    /// ```
    pub const fn new() -> Self {
        assert_nonzero_bitcount(BIT_COUNT);
        Self([0u8; BUCKET_COUNT])
    }

    /// Creates a new bitmap with all bits set to `true`.
    ///
    /// # Examples
    /// ```
    /// use light_bitmap::{bucket_count, BitMap};
    ///
    /// let bitmap = BitMap::<10, { bucket_count(10) }>::with_all_set();
    /// assert_eq!(bitmap.popcount(), 10);
    /// ```
    #[inline]
    pub fn with_all_set() -> Self {
        assert_nonzero_bitcount(BIT_COUNT);
        let mut bm = Self([!0u8; BUCKET_COUNT]);
        bm.clean_unused_bits();
        bm
    }

    /// Constructs a bitmap from a boolean slice, where `true` means set.
    ///
    /// # Panics
    /// Panics if the slice length doesn't match `BIT_COUNT`.
    ///
    /// # Examples
    /// ```
    /// use light_bitmap::{bucket_count, BitMap};
    ///
    /// let bitmap = BitMap::<4, { bucket_count(4) }>::from_slice(&[true, false, true, false]);
    /// assert_eq!(bitmap.popcount(), 2);
    /// ```
    #[inline]
    pub fn from_slice(bits: &[bool]) -> Self {
        assert_nonzero_bitcount(BIT_COUNT);
        assert_eq!(bits.len(), BIT_COUNT);
        let mut bm = Self([0u8; BUCKET_COUNT]);
        for (idx, bit) in bits.iter().enumerate() {
            if *bit {
                bm.set(idx)
            }
        }
        bm
    }

    /// Constructs a bitmap by setting only the indices provided in the iterator.
    ///
    /// All unspecified indices are left unset.
    ///
    /// # Panics
    /// Panics if any index is out of bounds (i.e., `>= BIT_COUNT`).
    ///
    /// # Examples
    /// ```
    /// use light_bitmap::{bucket_count, BitMap};
    ///
    /// let bitmap = BitMap::<5, { bucket_count(5) }>::from_ones_iter([0, 2, 4]);
    /// assert!(bitmap.is_set(0));
    /// assert!(!bitmap.is_set(1));
    /// assert_eq!(bitmap.popcount(), 3);
    /// ```
    pub fn from_ones_iter<I: IntoIterator<Item = usize>>(iter: I) -> Self {
        assert_nonzero_bitcount(BIT_COUNT);
        let mut bitmap = Self::new();
        for idx in iter {
            assert!(idx < BIT_COUNT, "Bit index {idx} out of bounds");
            bitmap.set(idx);
        }
        bitmap
    }

    /// Sets the bit at the given index to `true`.
    ///
    /// # Panics
    /// Panics if the index is out of bounds (i.e., `>= BIT_COUNT`).
    ///
    /// # Examples
    /// ```
    /// use light_bitmap::{bucket_count, BitMap};
    ///
    /// let mut bm = BitMap::<8, { bucket_count(8) }>::new();
    /// assert!(!bm.is_set(3));
    /// bm.set(3);
    /// assert!(bm.is_set(3));
    /// ```
    #[inline]
    pub fn set(&mut self, idx: usize) {
        assert!(idx < BIT_COUNT, "Bit index {idx} out of bounds");
        let (group_idx, item_idx) = Self::idxs(idx);
        self.0[group_idx] |= 1 << item_idx;
    }

    /// Sets all bits in the given range to `true`.
    ///
    /// # Panics
    /// Panics if `range.start >= BIT_COUNT` or `range.end > BIT_COUNT`.
    ///
    /// # Examples
    /// ```
    /// use light_bitmap::{bucket_count, BitMap};
    ///
    /// let mut bm = BitMap::<8, { bucket_count(8) }>::new();
    /// bm.set_range(2..6);
    /// assert!(bm.is_set(2));
    /// assert!(bm.is_set(5));
    /// assert!(!bm.is_set(6));
    /// ```
    pub fn set_range(&mut self, range: Range<usize>) {
        assert!(
            range.start < BIT_COUNT,
            "Range start {} out of bounds",
            range.start
        );
        assert!(
            range.end <= BIT_COUNT,
            "Range end {} out of bounds",
            range.end
        );

        if range.start >= range.end {
            return;
        }

        let (start_byte, start_bit) = Self::idxs(range.start);
        let (end_byte, end_bit) = Self::idxs(range.end - 1);

        // all within one byte
        if start_byte == end_byte {
            let width = end_bit - start_bit + 1;
            let mask = ((1u8 << width) - 1) << start_bit;
            self.0[start_byte] |= mask;
            return;
        }

        // set bits in first byte
        let first_mask = !0u8 << start_bit;
        self.0[start_byte] |= first_mask;

        // set full bytes in between
        for byte in &mut self.0[start_byte + 1..end_byte] {
            *byte = !0;
        }

        // set bits in last byte
        let last_mask = (1u8 << (end_bit + 1)) - 1;
        self.0[end_byte] |= last_mask;
    }

    /// Sets the bit at the given index to `false`.
    ///
    /// # Panics
    /// Panics if `idx >= BIT_COUNT`.
    ///
    /// # Examples
    /// ```
    /// use light_bitmap::{bucket_count, BitMap};
    ///
    /// let mut bm = BitMap::<8, { bucket_count(8) }>::with_all_set();
    /// assert!(bm.is_set(3));
    /// bm.unset(3);
    /// assert!(!bm.is_set(3));
    /// ```
    #[inline]
    pub fn unset(&mut self, idx: usize) {
        assert!(idx < BIT_COUNT, "Bit index {idx} out of bounds");
        let (group_idx, item_idx) = Self::idxs(idx);
        self.0[group_idx] &= !(1 << item_idx);
    }

    /// Clears all bits in the given range (sets them to `false`).
    ///
    /// # Panics
    /// Panics if `range.start >= BIT_COUNT` or `range.end > BIT_COUNT`.
    ///
    /// # Examples
    /// ```
    /// use light_bitmap::{bucket_count, BitMap};
    ///
    /// let mut bm = BitMap::<8, { bucket_count(8) }>::with_all_set();
    /// bm.unset_range(2..6);
    /// assert!(!bm.is_set(2));
    /// assert!(!bm.is_set(5));
    /// assert!(bm.is_set(6));
    /// ```
    pub fn unset_range(&mut self, range: Range<usize>) {
        assert!(
            range.start < BIT_COUNT,
            "Range start {} out of bounds",
            range.start
        );
        assert!(
            range.end <= BIT_COUNT,
            "Range end {} out of bounds",
            range.end
        );

        if range.start >= range.end {
            return;
        }

        let (start_byte, start_bit) = Self::idxs(range.start);
        let (end_byte, end_bit) = Self::idxs(range.end - 1);

        // all within one byte
        if start_byte == end_byte {
            let width = end_bit - start_bit + 1;
            let mask = !(((1u8 << width) - 1) << start_bit);
            self.0[start_byte] &= mask;
            return;
        }

        // set bits in first byte
        let first_mask = (1u8 << start_bit) - 1;
        self.0[start_byte] &= first_mask;

        // set full bytes in between
        for byte in &mut self.0[start_byte + 1..end_byte] {
            *byte = 0;
        }

        // set bits in last byte
        let last_mask = !((1u8 << (end_bit + 1)) - 1);
        self.0[end_byte] &= last_mask;
    }

    /// Toggles the bit at the given index.
    ///
    /// Returns the previous value of the bit (before the toggle).
    ///
    /// # Panics
    /// Panics if `idx >= BIT_COUNT`.
    ///
    /// # Examples
    /// ```
    /// use light_bitmap::{bucket_count, BitMap};
    ///
    /// let mut bm = BitMap::<8, { bucket_count(8) }>::new();
    /// assert_eq!(bm.toggle(4), false); // flipped from false to true
    /// assert_eq!(bm.toggle(4), true);  // flipped from true to false
    /// ```
    #[inline]
    pub fn toggle(&mut self, idx: usize) -> bool {
        assert!(idx < BIT_COUNT, "Bit index {idx} out of bounds");
        let (group_idx, item_idx) = Self::idxs(idx);
        let bit = self.0[group_idx] & 1 << item_idx != 0;
        self.0[group_idx] ^= 1 << item_idx;
        bit
    }

    /// Returns `true` if the bit at the given index is set.
    ///
    /// # Panics
    /// Panics if `idx >= BIT_COUNT`.
    ///
    /// # Examples
    /// ```
    /// use light_bitmap::{bucket_count, BitMap};
    ///
    /// let mut bm = BitMap::<8, { bucket_count(8) }>::new();
    /// bm.set(1);
    /// assert!(bm.is_set(1));
    /// assert!(!bm.is_set(0));
    /// ```
    #[inline]
    pub fn is_set(&self, idx: usize) -> bool {
        assert!(idx < BIT_COUNT, "Bit index {idx} out of bounds");
        let (group_idx, item_idx) = Self::idxs(idx);
        self.0[group_idx] & 1 << item_idx != 0
    }

    #[inline]
    fn idxs(idx: usize) -> (usize, usize) {
        (idx / 8, idx % 8)
    }

    /// Returns an iterator over all bits as `bool`, from least to most significant.
    ///
    /// The iterator yields exactly `BIT_COUNT` items in order.
    ///
    /// # Examples
    /// ```
    /// use light_bitmap::{bucket_count, BitMap};
    /// use core::array::from_fn;
    ///
    ///
    /// let bm = BitMap::<4, { bucket_count(4) }>::from_slice(&[true, false, true, false]);
    /// let mut bm_iter = bm.iter();
    /// assert_eq!(from_fn(|_| bm_iter.next().unwrap()), [true, false, true, false]);
    /// ```
    #[inline]
    pub fn iter(&self) -> BitMapIter<BIT_COUNT, BUCKET_COUNT> {
        BitMapIter {
            bytes: &self.0,
            group_idx: 0,
            item_idx: 0,
        }
    }

    /// Returns an iterator over the indices of all set bits (`true`), in ascending
    /// order.
    ///
    /// The iterator yields up to `BIT_COUNT` indices and is guaranteed to terminate.
    /// Iterating through the entire iterator runs in is O(min(k, b)) where k is
    /// the number of set bits and b is the number of buckets.
    ///
    /// # Examples
    /// ```
    /// use light_bitmap::{bucket_count, BitMap};
    /// use core::array::from_fn;
    ///
    /// let bm = BitMap::<5, { bucket_count(5) }>::from_slice(&[true, false, true, false, true]);
    /// let mut ones_iter = bm.iter_ones();
    /// let ones = from_fn(|i| ones_iter.next().unwrap_or(999));
    /// assert_eq!(ones, [0, 2, 4, 999, 999]);
    /// ```
    #[inline]
    pub fn iter_ones(&self) -> IterOnes<BIT_COUNT, BUCKET_COUNT> {
        IterOnes {
            bytes: &self.0,
            byte_idx: 0,
            current: self.0[0],
            base_bit_idx: 0,
        }
    }

    /// Returns an iterator over the indices of all unset bits (`false`), in ascending order.
    ///
    /// The iterator yields up to `BIT_COUNT` indices and is guaranteed to terminate.
    /// Iterating through the entire iterator runs in is O(min(k, b)) where k is
    /// the number of unset bits and b is the number of buckets.
    ///
    /// # Examples
    /// ```
    /// use light_bitmap::{bucket_count, BitMap};
    /// use core::array::from_fn;
    ///
    /// let bm = BitMap::<5, { bucket_count(5) }>::from_slice(&[true, false, true, false, true]);
    /// let mut zeros_iter = bm.iter_zeros();
    /// let zeros = from_fn(|i| zeros_iter.next().unwrap_or(999));
    /// assert_eq!(zeros, [1, 3, 999, 999, 999]);
    /// ```
    #[inline]
    pub fn iter_zeros(&self) -> IterZeros<BIT_COUNT, BUCKET_COUNT> {
        IterZeros {
            bytes: &self.0,
            byte_idx: 0,
            current: !self.0[0],
            base_bit_idx: 0,
        }
    }

    /// Returns a new bitmap representing the bitwise OR of `self` and `other`.
    ///
    /// Each bit in the result is set if it is set in either operand.
    ///
    /// # Examples
    /// ```
    /// use light_bitmap::{bucket_count, BitMap};
    ///
    /// let a = BitMap::<4, { bucket_count(4) }>::from_slice(&[true, false, true, false]);
    /// let b = BitMap::<4, { bucket_count(4) }>::from_slice(&[false, true, true, false]);
    /// let c = a.bit_or(&b);
    /// assert_eq!(c, BitMap::<4, { bucket_count(4) }>::from_slice(&[true, true, true, false]));
    /// ```
    #[inline]
    pub fn bit_or(&self, other: &Self) -> Self {
        Self(from_fn(|i| self.0[i] | other.0[i]))
    }

    /// Performs an in-place bitwise OR with another bitmap.
    ///
    /// Each bit in `self` is updated to the result of `self | other`.
    ///
    /// # Examples
    /// ```
    /// use light_bitmap::{bucket_count, BitMap};
    ///
    /// let mut a = BitMap::<4, { bucket_count(4) }>::from_slice(&[true, false, true, false]);
    /// let b = BitMap::<4, { bucket_count(4) }>::from_slice(&[false, true, true, false]);
    /// a.in_place_bit_or(&b);
    /// assert_eq!(a, BitMap::<4, { bucket_count(4) }>::from_slice(&[true, true, true, false]));
    /// ```
    #[inline]
    pub fn in_place_bit_or(&mut self, other: &Self) {
        for (self_byte, other_byte) in self.0.iter_mut().zip(other.0.iter()) {
            *self_byte |= other_byte
        }
    }

    /// Returns a new bitmap representing the bitwise AND of `self` and `other`.
    ///
    /// Each bit in the result is set only if it is set in both operands.
    ///
    /// # Examples
    /// ```
    /// use light_bitmap::{bucket_count, BitMap};
    ///
    /// let a = BitMap::<4, { bucket_count(4) }>::from_slice(&[true, false, true, false]);
    /// let b = BitMap::<4, { bucket_count(4) }>::from_slice(&[false, true, true, false]);
    /// let c = a.bit_and(&b);
    /// assert_eq!(c, BitMap::<4, { bucket_count(4) }>::from_slice(&[false, false, true, false]));
    /// ```
    #[inline]
    pub fn bit_and(&self, other: &Self) -> Self {
        Self(from_fn(|i| self.0[i] & other.0[i]))
    }

    /// Performs an in-place bitwise AND with another bitmap.
    ///
    /// Each bit in `self` is updated to the result of `self & other`.
    ///
    /// # Examples
    /// ```
    /// use light_bitmap::{bucket_count, BitMap};
    ///
    /// let mut a = BitMap::<4, { bucket_count(4) }>::from_slice(&[true, false, true, false]);
    /// let b = BitMap::<4, { bucket_count(4) }>::from_slice(&[false, true, true, false]);
    /// a.in_place_bit_and(&b);
    /// assert_eq!(a, BitMap::<4, { bucket_count(4) }>::from_slice(&[false, false, true, false]));
    /// ```
    #[inline]
    pub fn in_place_bit_and(&mut self, other: &Self) {
        for (self_byte, other_byte) in self.0.iter_mut().zip(other.0.iter()) {
            *self_byte &= other_byte
        }
    }

    /// Returns a new bitmap representing the bitwise XOR of `self` and `other`.
    ///
    /// Each bit in the result is set if it differs between the operands.
    ///
    /// # Examples
    /// ```
    /// use light_bitmap::{bucket_count, BitMap};
    ///
    /// let a = BitMap::<4, { bucket_count(4) }>::from_slice(&[true, false, true, false]);
    /// let b = BitMap::<4, { bucket_count(4) }>::from_slice(&[false, true, true, false]);
    /// let c = a.bit_xor(&b);
    /// assert_eq!(c, BitMap::<4, { bucket_count(4) }>::from_slice(&[true, true, false, false]));
    /// ```
    #[inline]
    pub fn bit_xor(&self, other: &Self) -> Self {
        Self(from_fn(|i| self.0[i] ^ other.0[i]))
    }

    /// Performs an in-place bitwise XOR with another bitmap.
    ///
    /// Each bit in `self` is updated to the result of `self ^ other`.
    ///
    /// # Examples
    /// ```
    /// use light_bitmap::{bucket_count, BitMap};
    ///
    /// let mut a = BitMap::<4, { bucket_count(4) }>::from_slice(&[true, false, true, false]);
    /// let b = BitMap::<4, { bucket_count(4) }>::from_slice(&[false, true, true, false]);
    /// a.in_place_bit_xor(&b);
    /// assert_eq!(a, BitMap::<4, { bucket_count(4) }>::from_slice(&[true, true, false, false]));
    /// ```
    #[inline]
    pub fn in_place_bit_xor(&mut self, other: &Self) {
        for (self_byte, other_byte) in self.0.iter_mut().zip(other.0.iter()) {
            *self_byte ^= other_byte
        }
    }

    /// Returns a new bitmap with each bit inverted (bitwise NOT).
    ///
    /// Each bit in the result is the inverse of the corresponding bit in self.
    ///
    /// # Examples
    /// ```
    /// use light_bitmap::{bucket_count, BitMap};
    ///
    /// let a = BitMap::<4, { bucket_count(4) }>::from_slice(&[false, false, true, false]);
    /// let b = a.bit_not();
    /// assert_eq!(b, BitMap::<4, { bucket_count(4) }>::from_slice(&[true, true, false, true]));
    /// ```
    #[inline]
    pub fn bit_not(&self) -> Self {
        let mut result = Self(from_fn(|i| !self.0[i]));
        result.clean_unused_bits();
        result
    }

    /// Inverts each bit of the bitmap in-place (bitwise NOT).
    ///
    /// Each bit in `self` is updated to the inverse of the corresponding bit in
    /// self.
    ///
    /// # Examples
    /// ```
    /// use light_bitmap::{bucket_count, BitMap};
    ///
    /// let mut a = BitMap::<4, { bucket_count(4) }>::from_slice(&[true, false, true, false]);
    /// a.in_place_bit_not();
    /// assert_eq!(a, BitMap::<4, { bucket_count(4) }>::from_slice(&[false, true, false, true]));
    /// ```
    #[inline]
    pub fn in_place_bit_not(&mut self) {
        for byte in &mut self.0 {
            *byte = !*byte;
        }
        self.clean_unused_bits();
    }

    /// Returns the number of set bits (`true`) in the bitmap.
    ///
    /// # Examples
    /// ```
    /// use light_bitmap::{bucket_count, BitMap};
    ///
    /// let bm = BitMap::<4, { bucket_count(4) }>::from_slice(&[true, false, true, false]);
    /// assert_eq!(bm.popcount(), 2);
    /// ```
    #[inline]
    pub fn popcount(&self) -> usize {
        self.0.iter().map(|b| b.count_ones() as usize).sum()
    }

    /// Returns the index of the first set bit (`true`), if any.
    ///
    /// Bits are checked in ascending order from least to most significant.
    /// Returns `None` if all bits are unset. Runs in O(b) where b is the bucket
    /// count.
    ///
    /// # Examples
    /// ```
    /// use light_bitmap::{bucket_count, BitMap};
    ///
    /// let empty = BitMap::<4, { bucket_count(4) }>::new();
    /// assert_eq!(empty.first_set_bit(), None);
    ///
    /// let mut bm = BitMap::<4, { bucket_count(4) }>::new();
    /// bm.set(2);
    /// assert_eq!(bm.first_set_bit(), Some(2));
    /// ```
    pub fn first_set_bit(&self) -> Option<usize> {
        for (i, byte) in self.0.iter().enumerate() {
            if *byte != 0 {
                let bit = byte.trailing_zeros() as usize;
                return Some(i * 8 + bit);
            }
        }
        None
    }

    #[inline]
    fn clean_unused_bits(&mut self) {
        let bits_in_last = BIT_COUNT % 8;
        if bits_in_last != 0 {
            let mask = (1 << bits_in_last) - 1;
            self.0[BUCKET_COUNT - 1] &= mask;
        }
    }

    /// Does a left shift by `n` positions, filling with `false`. This means
    /// bits are shifted towards higher bit indices.
    ///
    /// Bits that are shifted beyond `BIT_COUNT` are lost.
    /// If `n >= BIT_COUNT`, the bitmap is cleared.
    ///
    /// # Examples
    /// ```
    /// use light_bitmap::{bucket_count, BitMap};
    ///
    /// let mut bm = BitMap::<4, { bucket_count(4) }>::from_slice(&[true, false, true, false]);
    /// bm.shift_left(1);
    /// assert_eq!(bm, BitMap::<4, { bucket_count(4) }>::from_slice(&[false, true, false, true]));
    /// ```
    pub fn shift_left(&mut self, n: usize) {
        if n >= BIT_COUNT {
            self.0.fill(0);
            return;
        }
        let byte_shift = n / 8;
        let bit_shift = n % 8;

        if byte_shift > 0 {
            for i in (byte_shift..BUCKET_COUNT).rev() {
                self.0[i] = self.0[i - byte_shift];
            }
            for i in 0..byte_shift {
                self.0[i] = 0;
            }
        }

        if bit_shift > 0 {
            for i in (0..BUCKET_COUNT).rev() {
                let high = *self.0.get(i.wrapping_sub(1)).unwrap_or(&0);
                self.0[i] <<= bit_shift;
                self.0[i] |= high >> (8 - bit_shift);
            }
        }

        self.clean_unused_bits();
    }

    /// Does a right shift by `n` positions, filling with `false`. This means
    /// bits are shifted towards lower bit indices.
    ///
    /// Bits that are shifted beyond index 0 are lost.
    /// If `n >= BIT_COUNT`, the bitmap is cleared.
    ///
    /// # Examples
    /// ```
    /// use light_bitmap::{bucket_count, BitMap};
    ///
    /// let mut bm = BitMap::<4, { bucket_count(4) }>::from_slice(&[true, false, true, false]);
    /// bm.shift_right(1);
    /// assert_eq!(bm, BitMap::<4, { bucket_count(4) }>::from_slice(&[false, true, false, false]));
    /// ```
    pub fn shift_right(&mut self, n: usize) {
        if n >= BIT_COUNT {
            self.0.fill(0);
            return;
        }
        self.clean_unused_bits();

        let byte_shift = n / 8;
        let bit_shift = n % 8;

        if byte_shift > 0 {
            for i in 0..BUCKET_COUNT - byte_shift {
                self.0[i] = self.0[i + byte_shift];
            }
            for i in byte_shift..BUCKET_COUNT {
                self.0[i] = 0;
            }
        }

        if bit_shift > 0 {
            for i in 0..BUCKET_COUNT {
                let low = *self.0.get(i.wrapping_add(1)).unwrap_or(&0);
                self.0[i] >>= bit_shift;
                self.0[i] |= low << (8 - bit_shift);
            }
        }
    }

    /// Rotates all bits in direction of higher bit indices by `n` positions.
    /// Bits shifted out are reinserted on the other side.
    ///
    /// # Examples
    /// ```
    /// use light_bitmap::{bucket_count, BitMap};
    ///
    /// let mut bm = BitMap::<4, { bucket_count(4) }>::from_slice(&[true, false, false, true]);
    /// bm.rotate_left(1);
    /// assert_eq!(bm, BitMap::<4, { bucket_count(4) }>::from_slice(&[true, true, false, false]));
    /// ```
    pub fn rotate_left(&mut self, n: usize) {
        assert_nonzero_bitcount(BIT_COUNT);
        if n % BIT_COUNT == 0 {
            return;
        }
        let n = n % BIT_COUNT;
        let mut prev = self.is_set((BIT_COUNT - n) % BIT_COUNT);
        let mut bit_idx = 0;
        let mut start_idx = 0;
        for _ in 0..BIT_COUNT {
            let temp = self.is_set(bit_idx);
            if prev {
                self.set(bit_idx)
            } else {
                self.unset(bit_idx);
            }
            prev = temp;
            bit_idx = (bit_idx + n) % BIT_COUNT;
            if bit_idx == start_idx {
                start_idx += 1;
                bit_idx += 1;
                prev = self.is_set((bit_idx + BIT_COUNT - n) % BIT_COUNT)
            }
        }
    }

    /// Rotates all bits in direction of lower bit indices by `n` positions.
    /// Bits shifted out are reinserted on the other side.
    ///
    /// # Examples
    /// ```
    /// use light_bitmap::{bucket_count, BitMap};
    ///
    /// let mut bm = BitMap::<4, { bucket_count(4) }>::from_slice(&[true, false, false, true]);
    /// bm.rotate_right(1);
    /// assert_eq!(bm, BitMap::<4, { bucket_count(4) }>::from_slice(&[false, false, true, true]));
    /// ```
    pub fn rotate_right(&mut self, n: usize) {
        self.rotate_left(BIT_COUNT - n % BIT_COUNT);
    }
}

impl<const BIT_COUNT: usize, const BUCKET_COUNT: usize> Default
    for BitMap<BIT_COUNT, BUCKET_COUNT>
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'bitmap, const BIT_COUNT: usize, const BUCKET_COUNT: usize> IntoIterator
    for &'bitmap BitMap<BIT_COUNT, BUCKET_COUNT>
{
    type Item = bool;
    type IntoIter = BitMapIter<'bitmap, BIT_COUNT, BUCKET_COUNT>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<const BIT_COUNT: usize, const BUCKET_COUNT: usize> Debug for BitMap<BIT_COUNT, BUCKET_COUNT> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "LSB -> ")?;
        for (i, bit) in self.iter().enumerate() {
            if i % 8 == 0 {
                write!(f, "{i}: ")?;
            }
            write!(f, "{}", if bit { '1' } else { '0' })?;
            if i % 8 == 7 && i < BUCKET_COUNT * 8 - 1 {
                write!(f, " ")?;
            }
        }
        write!(f, " <- MSB")?;
        Ok(())
    }
}

/// Constructs a bitmap from an iterator over `bool`s.
///
/// # Panics
///
/// Panics if the iterator yields more or fewer than `BIT_COUNT` elements.
impl<const BIT_COUNT: usize, const BUCKET_COUNT: usize> FromIterator<bool>
    for BitMap<BIT_COUNT, BUCKET_COUNT>
{
    fn from_iter<T: IntoIterator<Item = bool>>(iter: T) -> Self {
        assert_nonzero_bitcount(BIT_COUNT);
        let mut bm = Self::new();
        let mut idx = 0;

        for bit in iter {
            if idx >= BIT_COUNT {
                panic!("Iterator yielded more than {BIT_COUNT} elements");
            }
            if bit {
                bm.set(idx);
            }
            idx += 1;
        }

        if idx != BIT_COUNT {
            panic!("Iterator yielded fewer than {BIT_COUNT} elements");
        }

        bm
    }
}

impl<const BIT_COUNT: usize, const BUCKET_COUNT: usize> BitAnd for BitMap<BIT_COUNT, BUCKET_COUNT> {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        self.bit_and(&rhs)
    }
}

impl<const BIT_COUNT: usize, const BUCKET_COUNT: usize> BitAndAssign
    for BitMap<BIT_COUNT, BUCKET_COUNT>
{
    fn bitand_assign(&mut self, rhs: Self) {
        self.in_place_bit_and(&rhs)
    }
}

impl<const BIT_COUNT: usize, const BUCKET_COUNT: usize> BitOr for BitMap<BIT_COUNT, BUCKET_COUNT> {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        self.bit_or(&rhs)
    }
}

impl<const BIT_COUNT: usize, const BUCKET_COUNT: usize> BitOrAssign
    for BitMap<BIT_COUNT, BUCKET_COUNT>
{
    fn bitor_assign(&mut self, rhs: Self) {
        self.in_place_bit_or(&rhs)
    }
}

impl<const BIT_COUNT: usize, const BUCKET_COUNT: usize> BitXor for BitMap<BIT_COUNT, BUCKET_COUNT> {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        self.bit_xor(&rhs)
    }
}

impl<const BIT_COUNT: usize, const BUCKET_COUNT: usize> BitXorAssign
    for BitMap<BIT_COUNT, BUCKET_COUNT>
{
    fn bitxor_assign(&mut self, rhs: Self) {
        self.in_place_bit_xor(&rhs)
    }
}

impl<const BIT_COUNT: usize, const BUCKET_COUNT: usize> Not for BitMap<BIT_COUNT, BUCKET_COUNT> {
    type Output = Self;

    fn not(self) -> Self::Output {
        self.bit_not()
    }
}

impl<const BIT_COUNT: usize, const BUCKET_COUNT: usize> Shl<usize>
    for BitMap<BIT_COUNT, BUCKET_COUNT>
{
    type Output = Self;

    fn shl(mut self, rhs: usize) -> Self::Output {
        self.shift_left(rhs);
        self
    }
}

impl<const BIT_COUNT: usize, const BUCKET_COUNT: usize> ShlAssign<usize>
    for BitMap<BIT_COUNT, BUCKET_COUNT>
{
    fn shl_assign(&mut self, rhs: usize) {
        self.shift_left(rhs);
    }
}

impl<const BIT_COUNT: usize, const BUCKET_COUNT: usize> Shr<usize>
    for BitMap<BIT_COUNT, BUCKET_COUNT>
{
    type Output = Self;

    fn shr(mut self, rhs: usize) -> Self::Output {
        self.shift_right(rhs);
        self
    }
}

impl<const BIT_COUNT: usize, const BUCKET_COUNT: usize> ShrAssign<usize>
    for BitMap<BIT_COUNT, BUCKET_COUNT>
{
    fn shr_assign(&mut self, rhs: usize) {
        self.shift_right(rhs);
    }
}

#[derive(Clone, Copy)]
pub struct BitMapIter<'bitmap, const BIT_COUNT: usize, const BUCKET_COUNT: usize> {
    bytes: &'bitmap [u8; BUCKET_COUNT],
    group_idx: usize,
    item_idx: usize,
}

impl<const BIT_COUNT: usize, const BUCKET_COUNT: usize> Iterator
    for BitMapIter<'_, BIT_COUNT, BUCKET_COUNT>
{
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        let absolute_idx = self.group_idx * 8 + self.item_idx;
        if absolute_idx >= BIT_COUNT {
            return None;
        }
        let bit = self.bytes[self.group_idx] & 1 << self.item_idx;
        self.item_idx += 1;
        if self.item_idx == 8 {
            self.item_idx = 0;
            self.group_idx += 1;
        }
        Some(bit != 0)
    }
}

impl<const BIT_COUNT: usize, const BUCKET_COUNT: usize> FusedIterator
    for BitMapIter<'_, BIT_COUNT, BUCKET_COUNT>
{
}

#[derive(Clone, Copy)]
pub struct IterOnes<'bitmap, const BIT_COUNT: usize, const BUCKET_COUNT: usize> {
    bytes: &'bitmap [u8; BUCKET_COUNT],
    byte_idx: usize,
    current: u8,
    base_bit_idx: usize,
}

impl<const BIT_COUNT: usize, const BUCKET_COUNT: usize> Iterator
    for IterOnes<'_, BIT_COUNT, BUCKET_COUNT>
{
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        while self.byte_idx < BUCKET_COUNT {
            if self.current != 0 {
                let tz = self.current.trailing_zeros() as usize;
                let idx = self.base_bit_idx + tz;
                if idx >= BIT_COUNT {
                    return None;
                }
                self.current &= self.current - 1; // unset LSB
                return Some(idx);
            }

            self.byte_idx += 1;
            self.base_bit_idx += 8;
            self.current = *self.bytes.get(self.byte_idx).unwrap_or(&0);
        }
        None
    }
}

impl<const BIT_COUNT: usize, const BUCKET_COUNT: usize> FusedIterator
    for IterOnes<'_, BIT_COUNT, BUCKET_COUNT>
{
}

#[derive(Clone, Copy)]
pub struct IterZeros<'bitmap, const BIT_COUNT: usize, const BUCKET_COUNT: usize> {
    bytes: &'bitmap [u8; BUCKET_COUNT],
    byte_idx: usize,
    current: u8,
    base_bit_idx: usize,
}

impl<const BIT_COUNT: usize, const BUCKET_COUNT: usize> Iterator
    for IterZeros<'_, BIT_COUNT, BUCKET_COUNT>
{
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        while self.byte_idx < BUCKET_COUNT {
            if self.current != 0 {
                let tz = self.current.trailing_zeros() as usize;
                let idx = self.base_bit_idx + tz;
                if idx >= BIT_COUNT {
                    return None;
                }
                self.current &= self.current - 1; // unset LSB
                return Some(idx);
            }

            self.byte_idx += 1;
            self.base_bit_idx += 8;
            self.current = !*self.bytes.get(self.byte_idx).unwrap_or(&0);
        }
        None
    }
}

impl<const BIT_COUNT: usize, const BUCKET_COUNT: usize> FusedIterator
    for IterZeros<'_, BIT_COUNT, BUCKET_COUNT>
{
}
