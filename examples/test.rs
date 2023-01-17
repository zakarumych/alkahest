use alkahest::{access, serialize, Schema, Seq};

fn main() {
    #[derive(Schema)]
    pub struct TestStruct(pub u32, pub bool, pub Seq<u32>, pub Seq<Seq<u32>>);

    let mut bytes = [0u8; 105];

    let size = serialize::<TestStruct, _>(
        TestStructSerialize(11, true, [1, 2, 3], (0..3).map(|i| (i..i + 4))),
        &mut bytes,
    )
    .unwrap();

    let test = access::<TestStruct>(&bytes);

    println!("{:#?}", test.0);
    println!("{:#?}", test.1);
    println!("{:#?}", test.2);

    for d in test.3 {
        println!("{:#?}", d);
    }

    // #[derive(alkahest::Schema)]
    // #[allow(dead_code)]
    // enum TestEnum<T> {
    //     Foo,
    //     Bar(T),
    //     Baz { val: f32 },
    //     Fuss { val: T, var: alkahest::Seq<u32> },
    // }

    // alkahest::write::<TestEnum<u64>, _>(&mut data.bytes, TestEnumFussPack { val: 4, var: 0..4 });

    // let test = alkahest::read::<TestEnum<u64>>(&data.bytes);

    // match test {
    //     TestEnumUnpacked::Foo => println!("Foo"),
    //     TestEnumUnpacked::Bar(val) => println!("Bar({})", val),
    //     TestEnumUnpacked::Baz { val } => println!("Bar{{val: {}}}", val),
    //     TestEnumUnpacked::Fuss { val, var } => {
    //         println!(
    //             "Fuss{{val: {}, var_sum: {}}}",
    //             val,
    //             var.into_iter().sum::<u32>()
    //         )
    //     }
    // }
}
