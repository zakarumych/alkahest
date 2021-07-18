struct AlignedBytes<T, const N: usize> {
    _align: [T; 0],
    bytes: [u8; N],
}

fn aligned_bytes<T, const N: usize>(bytes: [u8; N]) -> AlignedBytes<T, N> {
    AlignedBytes { _align: [], bytes }
}

fn main() {
    use alkahest::*;

    #[derive(Schema)]
    struct TestStruct<T: Schema> {
        a: u32,
        b: T,
        c: Seq<u32>,
        d: Seq<Seq<u32>>,
    }

    let mut data = aligned_bytes::<u32, 1024>([0; 1024]);

    write::<TestStruct<f32>, _>(
        &mut data.bytes,
        TestStructPack {
            a: 11,
            b: 42.0,
            c: std::array::IntoIter::new([1, 2, 3]),
            d: (0..3).map(|i| (i..i + 4)),
        },
    );

    let test = read::<TestStruct<f32>>(&data.bytes);

    println!("{:#?}", test.a);
    println!("{:#?}", test.b);
    println!("{:#?}", test.c.as_slice()); // `as_slice` is available when items are `Pod` types.
    for d in test.d {
        println!("{:#?}", d.as_slice());
    }
}
