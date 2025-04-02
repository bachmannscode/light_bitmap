use super::*;

#[test]
fn test_new() {
    macro_rules! test_new_by_bit_count {
                ($($bit_count:expr),+ $(,)?) => {
                    $(
                        {
                            const BIT_COUNT: usize = $bit_count;
                            let bitmap = BitMap::<BIT_COUNT, {bucket_count(BIT_COUNT)}>::new();
                            let mut bm_iter = bitmap.iter();
                            let actual: [bool; BIT_COUNT] = from_fn(|_| bm_iter.next().unwrap());
                            let expected = [false; BIT_COUNT];
                            assert_eq!(actual, expected, "Failed for BIT_COUNT = {}", BIT_COUNT);
                        }
                    )+
                };
            }

    test_new_by_bit_count!(1, 17, 31, 32, 33, 45, 111, 127, 128, 129, 45342);
}

#[test]
fn test_default() {
    macro_rules! test_new_by_bit_count {
                ($($bit_count:expr),+ $(,)?) => {
                    $(
                        {
                            const BIT_COUNT: usize = $bit_count;
                            let bitmap = BitMap::<BIT_COUNT, {bucket_count(BIT_COUNT)}>::default();
                            let mut bm_iter = bitmap.iter();
                            let actual: [bool; BIT_COUNT] = from_fn(|_| bm_iter.next().unwrap());
                            let expected = [false; BIT_COUNT];
                            assert_eq!(actual, expected, "Failed for BIT_COUNT = {}", BIT_COUNT);
                        }
                    )+
                };
            }

    test_new_by_bit_count!(1, 17, 31, 32, 33, 45, 111, 127, 128, 129, 45342);
}

#[test]
fn test_with_all_set() {
    macro_rules! test_all_set_by_bit_count {
            ($($bit_count:expr),+ $(,)?) => {
                $(
                    {
                        const BIT_COUNT: usize = $bit_count;
                        let bitmap = BitMap::<BIT_COUNT, {bucket_count(BIT_COUNT)}>::with_all_set();
                        let mut bm_iter = bitmap.iter();
                        let actual: [bool; BIT_COUNT] = from_fn(|_| bm_iter.next().unwrap());
                        let expected = [true; BIT_COUNT];
                        assert_eq!(actual, expected, "Failed for BIT_COUNT = {}", BIT_COUNT);
                    }
                )+
            };
        }

    test_all_set_by_bit_count!(1, 7, 8, 9, 17, 31, 32, 33, 45, 111, 127, 128, 129, 45342);
}

#[test]
fn ui_tests() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/bitmap_zero.rs");
}

#[test]
fn test_from_slice() {
    const BIT_COUNT: usize = 17;
    let input = [
        true, false, true, false, false, true, false, true, // 0..8
        true, false, true, false, true, true, false, true, // 8..16
        true, // 16
    ];

    let bitmap = BitMap::<BIT_COUNT, { bucket_count(BIT_COUNT) }>::from_slice(&input);
    let mut iter = bitmap.iter();
    let roundtripped: [bool; BIT_COUNT] = from_fn(|_| iter.next().unwrap());

    assert_eq!(roundtripped, input);
}

#[test]
#[should_panic(expected = "assertion `left == right` failed")]
fn test_from_slice_wrong_length() {
    const BIT_COUNT: usize = 35;
    let invalid_input = [true; BIT_COUNT + 1];
    let _ = BitMap::<BIT_COUNT, { bucket_count(BIT_COUNT) }>::from_slice(&invalid_input);
}

#[test]
fn test_from_iter_and_fromiterator() {
    const BIT_COUNT: usize = 10;
    let input = [
        true, false, true, false, false, true, true, true, false, true,
    ];

    let from_iter: BitMap<BIT_COUNT, { bucket_count(BIT_COUNT) }> = input.into_iter().collect();

    let expected = BitMap::<BIT_COUNT, { bucket_count(BIT_COUNT) }>::from_slice(&input);
    assert_eq!(from_iter, expected);
}

#[test]
#[should_panic(expected = "yielded more than")]
fn test_from_iter_too_many() {
    const BIT_COUNT: usize = 10;
    let input = [true; 11];
    let _: BitMap<BIT_COUNT, { bucket_count(BIT_COUNT) }> = input.into_iter().collect();
}

#[test]
#[should_panic(expected = "yielded fewer than")]
fn test_from_iter_too_few() {
    const BIT_COUNT: usize = 10;
    let input = [true; 9];
    let _: BitMap<BIT_COUNT, { bucket_count(BIT_COUNT) }> = input.into_iter().collect();
}

#[test]
fn test_from_ones_iter() {
    const BIT_COUNT: usize = 10;
    let bitmap = BitMap::<BIT_COUNT, { bucket_count(BIT_COUNT) }>::from_ones_iter([0, 2, 5, 7, 9]);

    let mut iter = bitmap.iter();
    let actual = from_fn(|_| iter.next().unwrap());

    assert_eq!(
        actual,
        [true, false, true, false, false, true, false, true, false, true]
    );
}

#[test]
#[should_panic(expected = "Bit index 10 out of bounds")]
fn test_from_ones_iter_out_of_bounds() {
    const BIT_COUNT: usize = 10;
    let _ = BitMap::<BIT_COUNT, { bucket_count(BIT_COUNT) }>::from_ones_iter([0, 2, 10]);
}

#[test]
fn test_iter() {
    const BIT_COUNT: usize = 10;
    let bitmap = BitMap::<BIT_COUNT, { bucket_count(BIT_COUNT) }>::from_slice(&[
        true, false, true, false, false, true, false, true, false, true,
    ]);

    let mut iter = bitmap.iter();
    let all = from_fn(|_| iter.next().unwrap());
    let mut ones_iter = bitmap.iter_ones();
    let ones = from_fn(|_| ones_iter.next().unwrap());
    let mut zeros_iter = bitmap.iter_zeros();
    let zeros = from_fn(|_| zeros_iter.next().unwrap());

    assert_eq!(
        all,
        [true, false, true, false, false, true, false, true, false, true]
    );
    assert_eq!(ones, [0, 2, 5, 7, 9]);
    assert_eq!(zeros, [1, 3, 4, 6, 8]);
}

#[test]
fn test_fused_iter_behavior() {
    const BIT_COUNT: usize = 10;
    let bitmap = BitMap::<BIT_COUNT, { bucket_count(BIT_COUNT) }>::from_slice(&[
        true, false, true, false, false, true, false, true, false, true,
    ]);

    let mut iter = bitmap.iter();
    for _ in 0..BIT_COUNT {
        assert!(iter.next().is_some());
    }
    for _ in 0..30 {
        assert_eq!(iter.next(), None);
    }

    let mut ones_iter = bitmap.iter_ones();
    for _ in 0..bitmap.popcount() {
        assert!(ones_iter.next().is_some());
    }
    for _ in 0..30 {
        assert_eq!(ones_iter.next(), None);
    }

    let mut zeros_iter = bitmap.iter_zeros();
    for _ in 0..(BIT_COUNT - bitmap.popcount()) {
        assert!(zeros_iter.next().is_some());
    }
    for _ in 0..30 {
        assert_eq!(zeros_iter.next(), None);
    }
}

#[test]
fn test_set_and_unset() {
    const BIT_COUNT: usize = 35;
    const BUCKET_COUNT: usize = bucket_count(BIT_COUNT);
    let mut bitmap = BitMap::<BIT_COUNT, BUCKET_COUNT>::new();

    for idx in 0..BIT_COUNT {
        // Set it
        bitmap.set(idx);
        assert!(bitmap.is_set(idx), "Bit {} should be set", idx);

        // Unset it again
        bitmap.unset(idx);
        assert!(!bitmap.is_set(idx), "Bit {} should be unset", idx);
    }
}

#[test]
#[should_panic(expected = "Bit index 35 out of bounds")]
fn test_set_out_of_bounds() {
    const BIT_COUNT: usize = 35;
    let mut bitmap = BitMap::<BIT_COUNT, { bucket_count(BIT_COUNT) }>::new();
    bitmap.set(BIT_COUNT); // one past the end
}

#[test]
fn test_set_range() {
    const BIT_COUNT: usize = 16;
    let mut bitmap = BitMap::<BIT_COUNT, { bucket_count(BIT_COUNT) }>::new();

    bitmap.set_range(3..10); // should set bits 3 through 9

    let expected = from_fn(|i| (3..10).contains(&i));
    let mut bm_iter = bitmap.iter();
    let actual: [bool; BIT_COUNT] = from_fn(|_| bm_iter.next().unwrap());

    assert_eq!(actual, expected);
}

#[test]
fn test_set_empty_range() {
    const BIT_COUNT: usize = 8;
    let mut bitmap = BitMap::<BIT_COUNT, { bucket_count(BIT_COUNT) }>::new();

    bitmap.set_range(5..5); // empty range
    let expected = [false; BIT_COUNT];
    let mut bm_iter = bitmap.iter();
    let actual = from_fn(|_| bm_iter.next().unwrap());

    assert_eq!(actual, expected);
}

#[test]
#[should_panic(expected = "Range end")]
fn test_set_end_range_out_of_bounds() {
    const BIT_COUNT: usize = 8;
    let mut bitmap = BitMap::<BIT_COUNT, { bucket_count(BIT_COUNT) }>::new();

    bitmap.set_range(2..BIT_COUNT + 1); // out-of-bounds end
}

#[test]
#[should_panic(expected = "Range start")]
fn test_set_start_range_out_of_bounds() {
    const BIT_COUNT: usize = 8;
    let mut bitmap = BitMap::<BIT_COUNT, { bucket_count(BIT_COUNT) }>::new();

    bitmap.set_range(BIT_COUNT..BIT_COUNT); // start too large
}

#[test]
fn test_unset_range() {
    const BIT_COUNT: usize = 16;
    let mut bitmap = BitMap::<BIT_COUNT, { bucket_count(BIT_COUNT) }>::with_all_set();

    bitmap.unset_range(3..10); // should unset bits 3 through 9

    let expected = from_fn(|i| !(3..10).contains(&i));
    let mut bm_iter = bitmap.iter();
    let actual: [bool; BIT_COUNT] = from_fn(|_| bm_iter.next().unwrap());

    assert_eq!(actual, expected);
}

#[test]
fn test_unset_empty_range() {
    const BIT_COUNT: usize = 35;
    let mut bitmap = BitMap::<BIT_COUNT, { bucket_count(BIT_COUNT) }>::with_all_set();

    bitmap.unset_range(5..5); // empty range, should do nothing
    let expected = [true; BIT_COUNT];
    let mut bm_iter = bitmap.iter();
    let actual = from_fn(|_| bm_iter.next().unwrap());

    assert_eq!(actual, expected);
}

#[test]
#[should_panic(expected = "Range end")]
fn test_unset_end_range_out_of_bounds() {
    const BIT_COUNT: usize = 35;
    let mut bitmap = BitMap::<BIT_COUNT, { bucket_count(BIT_COUNT) }>::with_all_set();

    bitmap.unset_range(2..BIT_COUNT + 1);
}

#[test]
#[should_panic(expected = "Range start")]
fn test_unset_start_range_out_of_bounds() {
    const BIT_COUNT: usize = 35;
    let mut bitmap = BitMap::<BIT_COUNT, { bucket_count(BIT_COUNT) }>::with_all_set();

    bitmap.unset_range(BIT_COUNT..BIT_COUNT);
}

#[test]
fn test_toggle() {
    const BIT_COUNT: usize = 35;
    let mut bitmap = BitMap::<BIT_COUNT, { bucket_count(BIT_COUNT) }>::new();

    // Bit is initially false
    assert!(!bitmap.is_set(3));

    // First toggle sets it
    let was_set = bitmap.toggle(3);
    assert!(!was_set);
    assert!(bitmap.is_set(3));
    assert_eq!(bitmap.popcount(), 1);

    // Second toggle unsets it
    let was_set = bitmap.toggle(3);
    assert!(was_set);
    assert!(!bitmap.is_set(3));
    assert_eq!(bitmap.popcount(), 0);
}

#[test]
fn test_toggle_twice_equals_original() {
    const BIT_COUNT: usize = 20;
    let mut bitmap = BitMap::<BIT_COUNT, { bucket_count(BIT_COUNT) }>::with_all_set();
    let original = bitmap;

    for idx in 0..BIT_COUNT {
        bitmap.toggle(idx);
    }

    assert_eq!(bitmap, original.bit_not());

    for idx in 0..BIT_COUNT {
        bitmap.toggle(idx);
    }

    assert_eq!(bitmap, original);
}

#[test]
#[should_panic(expected = "Bit index")]
fn test_toggle_out_of_bounds() {
    const BIT_COUNT: usize = 35;
    let mut bitmap = BitMap::<BIT_COUNT, { bucket_count(BIT_COUNT) }>::new();
    bitmap.toggle(BIT_COUNT); // should panic
}

#[test]
fn test_bit_or() {
    const BIT_COUNT: usize = 11;
    let a = BitMap::<BIT_COUNT, { bucket_count(BIT_COUNT) }>::from_slice(&[
        true, false, true, false, false, false, false, false, true, true, false,
    ]);
    let b = BitMap::<BIT_COUNT, { bucket_count(BIT_COUNT) }>::from_slice(&[
        false, true, false, true, false, false, false, false, false, true, true,
    ]);

    let expected = BitMap::<BIT_COUNT, { bucket_count(BIT_COUNT) }>::from_slice(&[
        true, true, true, true, false, false, false, false, true, true, true,
    ]);
    let result = a.bit_or(&b);
    assert_eq!(result, expected);

    let mut c = a;
    c.in_place_bit_or(&b);
    assert_eq!(c, expected);
}

#[test]
fn test_bit_and() {
    const BIT_COUNT: usize = 11;
    let a = BitMap::<BIT_COUNT, { bucket_count(BIT_COUNT) }>::from_slice(&[
        true, false, true, false, false, false, false, false, true, true, false,
    ]);
    let b = BitMap::<BIT_COUNT, { bucket_count(BIT_COUNT) }>::from_slice(&[
        false, true, true, true, false, false, false, false, false, true, true,
    ]);

    let expected = BitMap::<BIT_COUNT, { bucket_count(BIT_COUNT) }>::from_slice(&[
        false, false, true, false, false, false, false, false, false, true, false,
    ]);
    let result = a.bit_and(&b);
    assert_eq!(result, expected);

    let mut c = a;
    c.in_place_bit_and(&b);
    assert_eq!(c, expected);
}

#[test]
fn test_bit_xor() {
    const BIT_COUNT: usize = 11;
    let a = BitMap::<BIT_COUNT, { bucket_count(BIT_COUNT) }>::from_slice(&[
        true, false, true, false, false, false, false, false, true, true, false,
    ]);
    let b = BitMap::<BIT_COUNT, { bucket_count(BIT_COUNT) }>::from_slice(&[
        false, true, true, true, false, false, false, false, false, true, true,
    ]);

    let expected = BitMap::<BIT_COUNT, { bucket_count(BIT_COUNT) }>::from_slice(&[
        true, true, false, true, false, false, false, false, true, false, true,
    ]);
    let result = a.bit_xor(&b);
    assert_eq!(result, expected);

    let mut c = a;
    c.in_place_bit_xor(&b);
    assert_eq!(c, expected);
}

#[test]
fn test_bit_not_all_unset() {
    const BIT_COUNT: usize = 20;
    let bitmap = BitMap::<BIT_COUNT, { bucket_count(BIT_COUNT) }>::new();

    let inverted = BitMap::<BIT_COUNT, { bucket_count(BIT_COUNT) }>::with_all_set();
    assert_eq!(bitmap.bit_not(), inverted);
}

#[test]
fn test_bit_not_all_set() {
    const BIT_COUNT: usize = 20;
    let bitmap = BitMap::<BIT_COUNT, { bucket_count(BIT_COUNT) }>::with_all_set();

    let inverted = BitMap::<BIT_COUNT, { bucket_count(BIT_COUNT) }>::new();
    assert_eq!(bitmap.bit_not(), inverted);
}

#[test]
fn test_bit_not_inverts_correctly() {
    const BIT_COUNT: usize = 20;
    let original = BitMap::<BIT_COUNT, { bucket_count(BIT_COUNT) }>::from_slice(&[
        true, false, true, false, true, false, false, false, false, true, false, true, false,
        false, false, true, true, true, false, true,
    ]);

    let inverted = BitMap::<BIT_COUNT, { bucket_count(BIT_COUNT) }>::from_slice(&[
        false, true, false, true, false, true, true, true, true, false, true, false, true, true,
        true, false, false, false, true, false,
    ]);

    assert_eq!(original.bit_not(), inverted);
}

#[test]
fn test_in_place_bit_not() {
    const BIT_COUNT: usize = 20;
    let original = BitMap::<BIT_COUNT, { bucket_count(BIT_COUNT) }>::from_slice(&[
        true, false, false, false, false, false, true, false, false, true, true, false, true, true,
        false, false, false, true, false, true,
    ]);

    let inverted = original.bit_not();

    let mut inverted_in_place = original;
    inverted_in_place.in_place_bit_not();

    assert_eq!(inverted_in_place, inverted);
}

#[test]
fn test_operator_traits() {
    const BIT_COUNT: usize = 20;
    let a = BitMap::<BIT_COUNT, { bucket_count(BIT_COUNT) }>::from_slice(&[
        true, false, true, false, false, true, false, true, true, false, true, false, true, true,
        false, true, true, false, false, false,
    ]);
    let b = BitMap::<BIT_COUNT, { bucket_count(BIT_COUNT) }>::from_slice(&[
        false, true, true, false, true, false, false, true, false, true, true, false, false, true,
        false, false, false, false, false, true,
    ]);

    // bit_and
    assert_eq!(a.bit_and(&b), a & b);
    let mut tmp = a;
    tmp &= b;
    assert_eq!(tmp, a & b);

    // bit_or
    assert_eq!(a.bit_or(&b), a | b);
    let mut tmp = a;
    tmp |= b;
    assert_eq!(tmp, a | b);

    // bit_xor
    assert_eq!(a.bit_xor(&b), a ^ b);
    let mut tmp = a;
    tmp ^= b;
    assert_eq!(tmp, a ^ b);

    // bit_not
    assert_eq!(a.bit_not(), !a);

    // shift_left
    let mut m1 = a;
    m1.shift_left(3);
    assert_eq!(m1, a << 3);
    let mut m2 = a;
    m2 <<= 3;
    assert_eq!(m1, m2);

    // shift_right
    let mut m1 = a;
    m1.shift_right(2);
    assert_eq!(m1, a >> 2);
    let mut m2 = a;
    m2 >>= 2;
    assert_eq!(m1, m2);
}

#[test]
fn test_popcount() {
    const BIT_COUNT: usize = 20;

    let a = BitMap::<BIT_COUNT, { bucket_count(BIT_COUNT) }>::new();
    assert_eq!(a.popcount(), 0);

    let b = BitMap::<BIT_COUNT, { bucket_count(BIT_COUNT) }>::with_all_set();
    assert_eq!(b.popcount(), BIT_COUNT);

    let c = BitMap::<BIT_COUNT, { bucket_count(BIT_COUNT) }>::from_slice(&[
        true, false, true, false, false, false, false, false, true, false, false, false, true,
        false, false, false, false, false, true, false,
    ]);
    assert_eq!(c.popcount(), 5);
}

#[test]
fn test_first_set_bit() {
    const BIT_COUNT: usize = 20;

    let a = BitMap::<BIT_COUNT, { bucket_count(BIT_COUNT) }>::new();
    assert_eq!(a.first_set_bit(), None);

    let b = BitMap::<BIT_COUNT, { bucket_count(BIT_COUNT) }>::from_slice(&[
        false, false, false, false, false, true, false, false, true, true, false, false, false,
        false, false, false, false, false, false, false,
    ]);
    assert_eq!(b.first_set_bit(), Some(5));

    let mut c = BitMap::<BIT_COUNT, { bucket_count(BIT_COUNT) }>::from_slice(&[false; 20]);
    c.set(19);
    assert_eq!(c.first_set_bit(), Some(19));
}

#[test]
fn test_shift_left() {
    const BIT_COUNT: usize = 20;
    let mut bitmap = BitMap::<BIT_COUNT, { bucket_count(BIT_COUNT) }>::new();

    bitmap.set(0); // set first bit
    bitmap.shift_left(1);
    assert!(bitmap.is_set(1));
    assert_eq!(bitmap.popcount(), 1);

    bitmap.shift_left(7);
    assert!(bitmap.is_set(8)); // moved across a byte boundary
    assert_eq!(bitmap.popcount(), 1);

    bitmap.shift_left(12);
    assert_eq!(bitmap.popcount(), 0); // overflowed and cleared
}

#[test]
fn test_shift_left_zero_bits() {
    const BIT_COUNT: usize = 20;
    let mut bitmap = BitMap::<BIT_COUNT, { bucket_count(BIT_COUNT) }>::new();

    bitmap.set(5);
    bitmap.shift_left(0);
    assert!(bitmap.is_set(5)); // no change
}

#[test]
fn test_shift_left_cleans_unused_bits() {
    const BIT_COUNT: usize = 9; // last 6 bits of the last byte are unused
    let mut bitmap = BitMap::<BIT_COUNT, { bucket_count(BIT_COUNT) }>::new();

    // This will end up in the last byte
    bitmap.set(8);
    bitmap.shift_left(1); // moves bit 1 to bit 8

    // Unused bits must be cleaned
    let raw = bitmap.0[1];
    assert_eq!(
        raw & !((1 << (BIT_COUNT % 8)) - 1),
        0,
        "Unused bits must be zero"
    );
}

#[test]
fn test_shift_right() {
    const BIT_COUNT: usize = 20;
    let mut bitmap = BitMap::<BIT_COUNT, { bucket_count(BIT_COUNT) }>::new();

    bitmap.set(19); // set last bit
    bitmap.shift_right(1);
    assert!(bitmap.is_set(18));
    assert_eq!(bitmap.popcount(), 1);

    bitmap.shift_right(7);
    assert!(bitmap.is_set(11)); // moved across a byte boundary
    assert_eq!(bitmap.popcount(), 1);

    bitmap.shift_right(12);
    assert_eq!(bitmap.popcount(), 0); // overflowed and cleared
}

#[test]
fn test_shift_right_zero_bits() {
    const BIT_COUNT: usize = 16;
    let mut bitmap = BitMap::<BIT_COUNT, { bucket_count(BIT_COUNT) }>::new();

    bitmap.set(10);
    bitmap.shift_right(0);
    assert!(bitmap.is_set(10)); // no change
}

#[test]
fn test_shift_right_skips_unused_bits() {
    const BIT_COUNT: usize = 9;
    let mut bitmap = BitMap::<BIT_COUNT, { bucket_count(BIT_COUNT) }>::new();

    bitmap.set(0);
    bitmap.shift_right(1); // now bit 8 is set

    let raw = bitmap.0[1];
    assert_eq!(
        raw & !((1 << (BIT_COUNT % 8)) - 1),
        0,
        "Unused bits must be zero"
    );
}

#[test]
fn test_rotate_left() {
    const BIT_COUNT: usize = 20;
    let mut bitmap = BitMap::<BIT_COUNT, { bucket_count(BIT_COUNT) }>::new();

    bitmap.set(18);
    bitmap.rotate_left(3);

    assert!(bitmap.is_set(1));
    assert!(!bitmap.is_set(18));
    assert_eq!(bitmap.popcount(), 1);

    bitmap.rotate_left(2);
    assert!(bitmap.is_set(3));
    assert!(!bitmap.is_set(1));
    assert_eq!(bitmap.popcount(), 1);
}

#[test]
fn test_rotate_left_cleans_unused_bits() {
    const BIT_COUNT: usize = 9;
    let mut bitmap = BitMap::<BIT_COUNT, { bucket_count(BIT_COUNT) }>::with_all_set();

    bitmap.rotate_left(7);

    let raw = bitmap.0[1];
    assert_eq!(
        raw & !((1 << (BIT_COUNT % 8)) - 1),
        0,
        "Unused bits must be zero"
    );
}

#[test]
fn test_rotate_right() {
    const BIT_COUNT: usize = 20;
    let mut bitmap = BitMap::<BIT_COUNT, { bucket_count(BIT_COUNT) }>::new();

    bitmap.set(1);
    bitmap.rotate_right(3);

    assert!(bitmap.is_set(18));
    assert!(!bitmap.is_set(1));
    assert_eq!(bitmap.popcount(), 1);

    bitmap.rotate_right(2);
    assert!(bitmap.is_set(16));
    assert!(!bitmap.is_set(18));
    assert_eq!(bitmap.popcount(), 1)
}

#[test]
fn test_rotate_right_cleans_unused_bits() {
    const BIT_COUNT: usize = 9;
    let mut bitmap = BitMap::<BIT_COUNT, { bucket_count(BIT_COUNT) }>::with_all_set();

    bitmap.rotate_right(7);

    let raw = bitmap.0[1];
    assert_eq!(
        raw & !((1 << (BIT_COUNT % 8)) - 1),
        0,
        "Unused bits must be zero"
    );
}

#[test]
fn test_rotate_full_cycle() {
    const BIT_COUNT: usize = 20;
    let original = BitMap::<BIT_COUNT, { bucket_count(BIT_COUNT) }>::from_slice(&[
        true, false, true, false, true, false, true, false, true, false, false, true, false, true,
        false, true, false, true, false, true,
    ]);

    let mut left_rot = original;
    left_rot.rotate_left(BIT_COUNT);
    assert_eq!(left_rot, original);

    let mut right_rot = original;
    right_rot.rotate_right(BIT_COUNT * 2);
    assert_eq!(right_rot, original);
}
