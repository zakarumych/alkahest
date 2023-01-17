use alkahest::{Schema, Seq};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

#[derive(Schema)]
#[allow(dead_code)] // Values of schema type are never constructed. TODO: Generate code to silence this working for enums.
enum TestSchema {
    Foo { a: u32, b: u32 },
    Bar { c: Seq<u32>, d: Seq<Seq<u32>> },
}

enum TestSchemaPack<'a> {
    Foo(TestSchemaFooPack<u32, u32>),
    Bar(TestSchemaBarPack<&'a [u32], &'a [Vec<u32>]>),
}

#[derive(serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize)]
enum TestData {
    Foo { a: u32, b: u32 },
    Bar { c: Vec<u32>, d: Vec<Vec<u32>> },
}

impl TestData {
    fn wrapper(&self) -> TestSchemaPack<'_> {
        match self {
            &TestData::Foo { a, b } => TestSchemaPack::Foo(TestSchemaFooPack { a, b }),
            TestData::Bar { c, d } => TestSchemaPack::Bar(TestSchemaBarPack { c, d }),
        }
    }
}

impl alkahest::Pack<TestSchema> for TestSchemaPack<'_> {
    fn pack(self, offset: usize, output: &mut [u8]) -> (alkahest::Packed<TestSchema>, usize) {
        match self {
            TestSchemaPack::Foo(foo) => foo.pack(offset, output),
            TestSchemaPack::Bar(bar) => bar.pack(offset, output),
        }
    }
}

fn ser_alkahest(bytes: &mut [u8], data: &TestData) {
    alkahest::write::<TestSchema, _>(bytes, data.wrapper());
}

fn ser_json(bytes: &mut [u8], data: &TestData) {
    serde_json::to_writer(bytes, data).unwrap();
}

fn ser_bincode(bytes: &mut [u8], data: &TestData) {
    bincode::serialize_into(bytes, data).unwrap();
}

fn ser_rkyv(bytes: &mut [u8], data: &TestData) {
    let mut ser = rkyv::ser::serializers::CompositeSerializer::new(
        rkyv::ser::serializers::BufferSerializer::new(bytes),
        rkyv::ser::serializers::HeapScratch::<1024>::new(),
        rkyv::ser::serializers::SharedSerializeMap::new(),
    );
    rkyv::Serialize::serialize(data, &mut ser).unwrap();
}

fn de_alkahest(bytes: &[u8]) {
    match alkahest::read::<TestSchema>(bytes) {
        TestSchemaUnpacked::Foo { a, b } => {
            black_box(a);
            black_box(b);
        }
        TestSchemaUnpacked::Bar { c, d } => {
            c.into_iter().for_each(|c| {
                black_box(c);
            });
            d.into_iter().for_each(|d| {
                d.into_iter().for_each(|d| {
                    black_box(d);
                })
            });
        }
    }
}

fn de_json(bytes: &[u8]) {
    match serde_json::from_slice::<TestData>(bytes).unwrap() {
        TestData::Foo { a, b } => {
            black_box(a);
            black_box(b);
        }
        TestData::Bar { c, d } => {
            c.into_iter().for_each(|c| {
                black_box(c);
            });
            d.into_iter().for_each(|d| {
                d.into_iter().for_each(|d| {
                    black_box(d);
                })
            });
        }
    }
}

fn de_bincode(bytes: &[u8]) {
    match bincode::access::<TestData>(bytes).unwrap() {
        TestData::Foo { a, b } => {
            black_box(a);
            black_box(b);
        }
        TestData::Bar { c, d } => {
            c.into_iter().for_each(|c| {
                black_box(c);
            });
            d.into_iter().for_each(|d| {
                d.into_iter().for_each(|d| {
                    black_box(d);
                })
            });
        }
    }
}

fn de_rkyv(bytes: &[u8]) {
    match unsafe { rkyv::archived_root::<TestData>(bytes) } {
        ArchivedTestData::Foo { a, b } => {
            black_box(a);
            black_box(b);
        }
        ArchivedTestData::Bar { c, d } => {
            c.into_iter().for_each(|c| {
                black_box(c);
            });
            d.into_iter().for_each(|d| {
                d.into_iter().for_each(|d| {
                    black_box(d);
                })
            });
        }
    }
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let data = TestData::Bar {
        c: vec![1, 2, 3],
        d: vec![vec![4, 5, 1]],
    };

    let mut storage = [0u32; 128];
    let bytes = bytemuck::bytes_of_mut(&mut storage);

    let alkahest_vec = alkahest_to_vec::<TestSchema, _>(data.wrapper());
    let json_vec = serde_json::to_vec(&data).unwrap();
    let bincode_vec = bincode::serialize(&data).unwrap();

    let mut rkyv_ser = rkyv::ser::serializers::AllocSerializer::<1024>::default();
    rkyv::ser::Serializer::serialize_value(&mut rkyv_ser, &data).unwrap();
    let len = rkyv::ser::Serializer::pos(&rkyv_ser);
    let rkyv_vec = rkyv_ser.into_serializer().into_inner()[..len].to_vec();

    c.bench_function("ser alkahest", |b| {
        b.iter(|| ser_alkahest(bytes, black_box(&data)))
    });

    c.bench_function("ser json", |b| b.iter(|| ser_json(bytes, black_box(&data))));

    c.bench_function("ser bincode", |b| {
        b.iter(|| ser_bincode(bytes, black_box(&data)))
    });

    c.bench_function("ser rkyv", |b| b.iter(|| ser_rkyv(bytes, black_box(&data))));

    c.bench_function("de alkahest", |b| {
        b.iter(|| de_alkahest(black_box(&alkahest_vec)))
    });

    c.bench_function("de json", |b| b.iter(|| de_json(black_box(&json_vec))));

    c.bench_function("de bincode", |b| {
        b.iter(|| de_bincode(black_box(&bincode_vec)))
    });

    c.bench_function("de rkyv", |b| b.iter(|| de_rkyv(black_box(&rkyv_vec))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

/// Writes the alkahest into provided bytes slice.
/// Returns number of bytes written.

pub fn alkahest_to_vec<'a, T, P>(wrapper: P) -> Vec<u8>
where
    T: Schema,
    P: alkahest::Pack<T>,
{
    struct Aligned {
        _align: [u128; 0],
        bytes: [u8; 1024],
    }

    let mut aligned = Aligned {
        _align: [],
        bytes: [0; 1024],
    };

    let size = alkahest::write(&mut aligned.bytes, wrapper);
    aligned.bytes[..size].to_vec()
}
