use alkahest::{Deserialize, Formula, Serialize, SliceIter};
use bytecheck::CheckBytes;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

#[derive(
    Formula,
    Serialize,
    Deserialize,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
)]
#[archive_attr(derive(CheckBytes))]
enum TestData {
    Foo { a: u32, b: u32 },
    Bar { c: Vec<u32>, d: Vec<Vec<u32>> },
}

#[derive(Deserialize)]
#[alkahest(for<'de2: 'de> TestData)]
enum TestDataLazy<'de> {
    Foo {
        a: u32,
        b: u32,
    },
    Bar {
        c: SliceIter<'de, u32>,
        d: SliceIter<'de, Vec<u32>, SliceIter<'de, u32>>,
    },
}

fn ser_alkahest(bytes: &mut [u8], data: &TestData) {
    alkahest::serialize::<TestData, _>(data, bytes).unwrap();
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
    match alkahest::deserialize::<TestData, TestDataLazy>(bytes)
        .unwrap()
        .0
    {
        TestDataLazy::Foo { a, b } => {
            black_box(a);
            black_box(b);
        }
        TestDataLazy::Bar { c, d } => {
            c.into_iter().for_each(|c| {
                black_box(c.unwrap());
            });
            d.into_iter().for_each(|d| {
                d.unwrap().into_iter().for_each(|d| {
                    black_box(d.unwrap());
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
    match bincode::deserialize::<TestData>(bytes).unwrap() {
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
    match rkyv::check_archived_root::<TestData>(bytes).unwrap() {
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

    let mut bytes = [0u8; 1024];

    let alkahest_vec = alkahest_to_vec::<TestData, _>(&data);
    let json_vec = serde_json::to_vec(&data).unwrap();
    let bincode_vec = bincode::serialize(&data).unwrap();

    let mut rkyv_ser = rkyv::ser::serializers::AllocSerializer::<1024>::default();
    rkyv::ser::Serializer::serialize_value(&mut rkyv_ser, &data).unwrap();
    let len = rkyv::ser::Serializer::pos(&rkyv_ser);
    let rkyv_vec = rkyv_ser.into_serializer().into_inner()[..len].to_vec();

    c.bench_function("ser/alkahest", |b| {
        b.iter(|| ser_alkahest(&mut bytes, black_box(&data)))
    });

    c.bench_function("ser/json", |b| {
        b.iter(|| ser_json(&mut bytes, black_box(&data)))
    });

    c.bench_function("ser/bincode", |b| {
        b.iter(|| ser_bincode(&mut bytes, black_box(&data)))
    });

    c.bench_function("ser/rkyv", |b| {
        b.iter(|| ser_rkyv(&mut bytes, black_box(&data)))
    });

    c.bench_function("de/alkahest", |b| {
        b.iter(|| de_alkahest(black_box(&alkahest_vec)))
    });

    c.bench_function("de/json", |b| b.iter(|| de_json(black_box(&json_vec))));

    c.bench_function("de/bincode", |b| {
        b.iter(|| de_bincode(black_box(&bincode_vec)))
    });

    c.bench_function("de/rkyv", |b| b.iter(|| de_rkyv(black_box(&rkyv_vec))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

/// Writes the alkahest into provided bytes slice.
/// Returns number of bytes written.

pub fn alkahest_to_vec<'de, T, P>(data: P) -> Vec<u8>
where
    T: Formula,
    P: alkahest::Serialize<T>,
{
    let mut bytes = Vec::new();
    bytes.resize(1024, 0);

    let size = alkahest::serialize(data, &mut bytes).unwrap();
    bytes[..size].to_vec()
}
