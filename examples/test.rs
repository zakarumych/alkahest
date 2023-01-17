use alkahest::{access, serialize, Schema, Seq};

fn main() {
    #[derive(Schema)]
    pub struct TestStruct<T: Schema> {
        pub a: u32,
        pub b: T,
        pub c: Seq<u32>,
        pub d: Seq<Seq<u32>>,
    }

    let mut bytes = [0u8; 105];

    let size = serialize::<TestStruct<bool>, _>(
        TestStructSerialize {
            a: 11,
            b: true,
            c: [1, 2, 3],
            d: (0..3).map(|i| (i..i + 4)),
        },
        &mut bytes,
    )
    .unwrap();

    let test = access::<TestStruct<bool>>(&bytes);

    println!("{:#?}", test.a);
    println!("{:#?}", test.b);
    println!("{:#?}", test.c);

    for d in test.d {
        println!("{:#?}", d);
    }

    #[derive(Schema)]
    #[allow(dead_code)]
    enum TestEnum<T: Schema> {
        Foo,
        Bar(T),
        Baz { val: f32 },
        Fuss { val: T, var: Seq<u32> },
    }

    let size = alkahest::serialize::<TestEnum<u64>, _>(
        TestEnumFussSerialize { val: 4, var: 0..4 },
        &mut bytes,
    )
    .unwrap();

    let test = alkahest::access::<TestEnum<u64>>(&bytes);

    match test {
        TestEnumAccess::Foo => println!("Foo"),
        TestEnumAccess::Bar(val) => println!("Bar({})", val),
        TestEnumAccess::Baz { val } => println!("Bar{{val: {}}}", val),
        TestEnumAccess::Fuss { val, var } => {
            println!("Fuss{{val: {}, var: {:?}}}", val, var)
        }
    }
}
