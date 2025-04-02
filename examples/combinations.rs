use light_bitmap::{bucket_count, BitMap};

fn combinations<const BIT_COUNT: usize, const BUCKET_COUNT: usize>(
    idx: usize,
    bitmap: &mut BitMap<BIT_COUNT, BUCKET_COUNT>,
    data: &mut [bool],
) {
    if idx == data.len() {
        for (idx, pick) in (0..data.len()).zip(bitmap.iter()) {
            data[idx] = pick;
        }
        println!("{data:?}");
        return;
    }
    if !bitmap.is_set(idx) {
        bitmap.set(idx);
        combinations(idx + 1, bitmap, data);
        bitmap.unset(idx)
    }
    combinations(idx + 1, bitmap, data)
}

fn main() {
    const BIT_COUNT: usize = 9;
    let mut bitmap = BitMap::<BIT_COUNT, { bucket_count(BIT_COUNT) }>::new();
    let mut my_data = [false; BIT_COUNT];
    combinations(0, &mut bitmap, &mut my_data);
}
