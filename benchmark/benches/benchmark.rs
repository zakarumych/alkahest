extern crate alkahest;
extern crate criterion;

#[cfg(feature = "rkyv")]
extern crate bytecheck;

use alkahest::{Deserialize, Formula, LazySlice, Ref, Serialize};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

#[cfg(feature = "rkyv")]
use bytecheck::CheckBytes;

#[derive(Formula)]
enum TestFormula {
    Foo { a: Ref<u32>, b: Ref<u32> },
    Bar { c: Vec<u32>, d: Vec<Vec<u32>> },
}

#[derive(Serialize, Deserialize)]
#[alkahest(TestFormula)]
#[cfg_attr(feature = "serde", serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "rkyv", rkyv::Archive, rkyv::Serialize)]
#[cfg_attr(feature = "rkyv", archive_attr(derive(CheckBytes)))]
enum TestData {
    Foo { a: u32, b: u32 },
    Bar { c: Vec<u32>, d: Vec<Vec<u32>> },
}

#[derive(Deserialize)]
#[alkahest(for<'de: 'a> TestFormula)]
enum TestDataLazy<'a> {
    Foo {
        a: u32,
        b: u32,
    },
    Bar {
        c: LazySlice<'a, u32>,
        d: LazySlice<'a, Vec<u32>, LazySlice<'a, u32>>,
    },
}

fn ser_alkahest(bytes: &mut [u8], data: &TestData) -> usize {
    alkahest::serialize::<TestFormula, _>(data, bytes).unwrap()
}

#[cfg(feature = "serde_json")]
fn ser_json(bytes: &mut [u8], data: &TestData) -> usize {
    serde_json::to_writer(bytes, data).unwrap();
}

#[cfg(feature = "bincode")]
fn ser_bincode(bytes: &mut [u8], data: &TestData) -> usize {
    bincode::serialize_into(bytes, data).unwrap();
}

#[cfg(feature = "rkyv")]
fn ser_rkyv(bytes: &mut [u8], data: &TestData) -> usize {
    let mut ser = rkyv::ser::serializers::CompositeSerializer::new(
        rkyv::ser::serializers::BufferSerializer::new(bytes),
        rkyv::ser::serializers::HeapScratch::<1024>::new(),
        rkyv::ser::serializers::SharedSerializeMap::new(),
    );
    rkyv::Serialize::serialize(data, &mut ser).unwrap();
}

fn de_alkahest(bytes: &[u8]) {
    match alkahest::deserialize::<TestFormula, TestDataLazy>(bytes)
        .unwrap()
        .0
    {
        TestDataLazy::Foo { a, b } => {
            black_box(a);
            black_box(b);
        }
        TestDataLazy::Bar { c, d } => {
            c.iter().for_each(|c| {
                black_box(c.unwrap());
            });
            d.iter().for_each(|d| {
                d.unwrap().into_iter().for_each(|d| {
                    black_box(d.unwrap());
                })
            });
        }
    }
}

#[cfg(feature = "serde_json")]
fn de_json(bytes: &[u8]) {
    match serde_json::from_slice::<TestData>(bytes).unwrap() {
        TestData::Foo { a, b } => {
            black_box(a);
            black_box(b);
        }
        TestData::Bar { c, d } => {
            c.iter().for_each(|c| {
                black_box(c);
            });
            d.iter().for_each(|d| {
                d.iter().for_each(|d| {
                    black_box(d);
                })
            });
        }
    }
}

#[cfg(feature = "bincode")]
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

#[cfg(feature = "rkyv")]
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
    // let data = TestData::Bar {
    //     c: vec![1, 2, 3],
    //     d: vec![vec![4, 5, 1]],
    // };

    let data = TestData::Foo { a: 1, b: 2 };

    let mut bytes = [0u8; 1024];
    let mut size = 0;

    c.bench_function("first/alkahest/ser", |b| {
        b.iter(|| size = ser_alkahest(&mut bytes, black_box(&data)))
    });

    c.bench_function("first/alkahest/de", |b| {
        b.iter(|| de_alkahest(black_box(&bytes[..size])))
    });

    #[cfg(feature = "serde_json")]
    {
        c.bench_function("first/json/ser", |b| {
            b.iter(|| size = ser_json(&mut bytes, black_box(&data)))
        });

        c.bench_function("first/json/de", |b| {
            b.iter(|| de_json(black_box(&bytes[..size])))
        });
    }

    #[cfg(feature = "bincode")]
    {
        c.bench_function("first/bincode/ser", |b| {
            b.iter(|| size = ser_bincode(&mut bytes, black_box(&data)))
        });

        c.bench_function("first/bincode/de", |b| {
            b.iter(|| de_bincode(black_box(&bytes[..size])))
        });
    }

    #[cfg(feature = "rkyv")]
    {
        c.bench_function("first/rkyv/ser", |b| {
            b.iter(|| size = ser_rkyv(&mut bytes, black_box(&data)))
        });

        c.bench_function("first/rkyv/de", |b| {
            b.iter(|| de_rkyv(black_box(&bytes[..size])))
        });
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
