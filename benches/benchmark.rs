use alkahest::{Seq, Serialize, UnsizedFormula};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

#[derive(Formula)]
#[allow(dead_code)] // Values of formula type are never constructed. TODO: Generate code to silence this working for enums.
enum TestFormula {
    Foo { a: u32, b: u32 },
    Bar { c: Seq<u32>, d: Seq<Seq<u32>> },
}

enum TestFormulaHeader<'a> {
    Foo(<TestFormulaFooSerialize<u32, u32> as Serialize<TestFormula>>::Header),
    Bar(<TestFormulaBarSerialize<&'a [u32], &'a [Vec<u32>]> as Serialize<TestFormula>>::Header),
}

#[derive(serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize)]
enum TestData {
    Foo { a: u32, b: u32 },
    Bar { c: Vec<u32>, d: Vec<Vec<u32>> },
}

impl<'a> alkahest::Serialize<TestFormula> for &'a TestData {
    type Header = TestFormulaHeader<'a>;

    fn serialize_body(self, output: &mut [u8]) -> Result<(Self::Header, usize), usize> {
        match *self {
            TestData::Foo { a, b } => {
                let (header, size) = TestFormulaFooSerialize { a, b }.serialize_body(output)?;
                Ok((TestFormulaHeader::Foo(header), size))
            }
            TestData::Bar { ref c, ref d } => {
                let (header, size) = TestFormulaBarSerialize { c, d }.serialize_body(output)?;
                Ok((TestFormulaHeader::Bar(header), size))
            }
        }
    }

    fn serialize_header(header: TestFormulaHeader<'a>, output: &mut [u8], offset: usize) -> bool {
        match header {
            TestFormulaHeader::Foo(header) => {
                TestFormulaFooSerialize::<u32, u32>::serialize_header(header, output, offset)
            }
            TestFormulaHeader::Bar(header) => {
                TestFormulaBarSerialize::<&'a [u32], &'a [Vec<u32>]>::serialize_header(
                    header, output, offset,
                )
            }
        }
    }
}

fn ser_alkahest(bytes: &mut [u8], data: &TestData) {
    alkahest::serialize::<TestFormula, _>(data, bytes).unwrap();
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
    match alkahest::access::<TestFormula>(bytes) {
        TestFormulaAccess::Foo { a, b } => {
            black_box(a);
            black_box(b);
        }
        TestFormulaAccess::Bar { c, d } => {
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

    let mut bytes = [0u8; 1024];

    let alkahest_vec = alkahest_to_vec::<TestFormula, _>(&data);
    let json_vec = serde_json::to_vec(&data).unwrap();
    let bincode_vec = bincode::serialize(&data).unwrap();

    let mut rkyv_ser = rkyv::ser::serializers::AllocSerializer::<1024>::default();
    rkyv::ser::Serializer::serialize_value(&mut rkyv_ser, &data).unwrap();
    let len = rkyv::ser::Serializer::pos(&rkyv_ser);
    let rkyv_vec = rkyv_ser.into_serializer().into_inner()[..len].to_vec();

    c.bench_function("ser alkahest", |b| {
        b.iter(|| ser_alkahest(&mut bytes, black_box(&data)))
    });

    c.bench_function("ser json", |b| {
        b.iter(|| ser_json(&mut bytes, black_box(&data)))
    });

    c.bench_function("ser bincode", |b| {
        b.iter(|| ser_bincode(&mut bytes, black_box(&data)))
    });

    c.bench_function("ser rkyv", |b| {
        b.iter(|| ser_rkyv(&mut bytes, black_box(&data)))
    });

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

pub fn alkahest_to_vec<'a, T, P>(data: P) -> Vec<u8>
where
    T: UnsizedFormula,
    P: alkahest::Serialize<T>,
{
    let mut bytes = Vec::new();
    bytes.resize(1024, 0);

    let size = alkahest::serialize(data, &mut bytes).unwrap();
    bytes[..size].to_vec()
}
