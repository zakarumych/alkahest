struct AlignedBytes<T, const N: usize> {
    _align: [T; 0],
    bytes: [u8; N],
}

fn aligned_bytes<T, const N: usize>(bytes: [u8; N]) -> AlignedBytes<T, N> {
    AlignedBytes { _align: [], bytes }
}

fn main() {
    #[derive(alkahest::Schema)]
    struct TestStruct<T: alkahest::Schema> {
        a: u32,
        b: T,
        c: alkahest::Seq<u32>,
        d: alkahest::Seq<alkahest::Seq<u32>>,
    }

    let mut data = aligned_bytes::<u32, 1024>([0; 1024]);

    alkahest::write::<TestStruct<f32>, _>(
        &mut data.bytes,
        TestStructPack {
            a: 11,
            b: 42.0,
            c: std::array::IntoIter::new([1, 2, 3]),
            d: (0..3).map(|i| (i..i + 4)),
        },
    );

    let test = alkahest::read::<TestStruct<f32>>(&data.bytes);

    println!("{:#?}", test.a);
    println!("{:#?}", test.b);
    println!("{:#?}", test.c.as_slice()); // `as_slice` is available when items are `Pod` types.
    for d in test.d {
        println!("{:#?}", d.as_slice());
    }

    #[derive(alkahest::Schema)]
    #[allow(dead_code)]
    enum TestEnum<T: alkahest::Schema> {
        Foo,
        Bar(T),
        Baz { val: f32 },
        Fuss { val: T, var: alkahest::Seq<u32> },
    }

    alkahest::write::<TestEnum<u64>, _>(&mut data.bytes, TestEnumFussPack { val: 4, var: 0..4 });

    let test = alkahest::read::<TestEnum<u64>>(&data.bytes);

    match test {
        TestEnumUnpacked::Foo => println!("Foo"),
        TestEnumUnpacked::Bar(val) => println!("Bar({})", val),
        TestEnumUnpacked::Baz { val } => println!("Bar{{val: {}}}", val),
        TestEnumUnpacked::Fuss { val, var } => {
            println!(
                "Fuss{{val: {}, var_sum: {}}}",
                val,
                var.into_iter().sum::<u32>()
            )
        }
    }
}
