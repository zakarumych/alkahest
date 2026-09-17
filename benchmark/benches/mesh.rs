use std::hint::black_box;

use alkahest::{Lazy, List, Mixture, alkahest};
use criterion::Criterion;

#[cfg(feature = "rkyv")]
use bytecheck::CheckBytes;
use rand::{
    Rng, RngExt, SeedableRng,
    distr::{Distribution, StandardUniform},
    rngs::SmallRng,
};

#[derive(Debug, Clone, Copy, PartialEq, Mixture)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
#[cfg_attr(feature = "rkyv", archive_attr(derive(CheckBytes)))]
#[cfg_attr(feature = "speedy", derive(speedy::Writable, speedy::Readable))]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Distribution<Vector3> for StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Vector3 {
        Vector3 {
            x: rng.random(),
            y: rng.random(),
            z: rng.random(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Mixture)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
#[cfg_attr(feature = "rkyv", archive_attr(derive(CheckBytes)))]
#[cfg_attr(feature = "speedy", derive(speedy::Writable, speedy::Readable))]
pub struct Triangle {
    pub v0: Vector3,
    pub v1: Vector3,
    pub v2: Vector3,
    pub normal: Vector3,
}

impl Distribution<Triangle> for StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Triangle {
        let v0 = rng.random();
        let v1 = rng.random();
        let v2 = rng.random();
        let normal = rng.random();

        Triangle { v0, v1, v2, normal }
    }
}

#[alkahest(Formula)]
pub struct MeshFormula {
    pub triangles: List<Triangle>,
}

#[derive(Debug, Clone, PartialEq)]
#[alkahest(Serialize<MeshFormula>)]
#[alkahest(Deserialize<MeshFormula>)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
#[cfg_attr(feature = "rkyv", archive_attr(derive(CheckBytes)))]
#[cfg_attr(feature = "speedy", derive(speedy::Writable, speedy::Readable))]
pub struct Mesh {
    pub triangles: Vec<Triangle>,
}

#[alkahest(Deserialize<'a, MeshFormula>)]
pub struct LazyMesh<'a> {
    pub triangles: Lazy<'a, List<Triangle>>,
}

macro_rules! observe_vector {
    ($vector:expr) => {{
        let vector = $vector;
        black_box(vector.x);
        black_box(vector.y);
        black_box(vector.z);
    }};
}

macro_rules! observe_triangle {
    ($triangle:expr) => {{
        let triangle = $triangle;
        observe_vector!(&triangle.v0);
        observe_vector!(&triangle.v1);
        observe_vector!(&triangle.v2);
        observe_vector!(&triangle.normal);
    }};
}

fn observe(mesh: &Mesh) {
    for triangle in &mesh.triangles {
        observe_triangle!(triangle);
    }
}

pub fn criterion_benchmark(c: &mut Criterion) {
    const TRIANGLE_COUNT: usize = 100_000;
    let mut rng = SmallRng::seed_from_u64(42);
    let mesh = Mesh {
        triangles: (&mut rng).random_iter().take(TRIANGLE_COUNT).collect(),
    };

    {
        let mut group = c.benchmark_group("mesh/alkahest");
        let mut encoded = Vec::new();
        let size = alkahest::serialize_to_vec::<MeshFormula, _>(&mesh, &mut encoded);
        let encoded = &encoded[..size];
        assert_eq!(
            alkahest::deserialize::<MeshFormula, Mesh>(encoded).expect("valid mesh"),
            mesh
        );
        let read =
            alkahest::deserialize::<MeshFormula, LazyMesh>(encoded).expect("valid lazy mesh");
        assert_eq!(
            read.triangles
                .iter::<Triangle>()
                .expect("valid triangle list")
                .collect::<Result<Vec<_>, _>>()
                .expect("valid triangles"),
            mesh.triangles
        );

        let mut output = Vec::with_capacity(size);
        group.bench_function("serialize", |b| {
            b.iter(|| {
                black_box(alkahest::serialize_to_vec::<MeshFormula, _>(
                    black_box(&mesh),
                    &mut output,
                ));
                black_box(&output);
            })
        });
        group.bench_function("read", |b| {
            b.iter(|| {
                let read = alkahest::deserialize::<MeshFormula, LazyMesh>(black_box(encoded))
                    .expect("valid lazy mesh");
                for triangle in read
                    .triangles
                    .iter::<Triangle>()
                    .expect("valid triangle list")
                {
                    observe_triangle!(&triangle.expect("valid triangle"));
                }
            })
        });
        group.bench_function("deserialize", |b| {
            b.iter(|| {
                let read = alkahest::deserialize::<MeshFormula, Mesh>(black_box(encoded))
                    .expect("valid mesh");
                observe(&read);
            })
        });
    }

    #[cfg(feature = "bincode")]
    {
        let mut group = c.benchmark_group("mesh/bincode");
        let encoded = bincode::serialize(&mesh).expect("serializable mesh");
        assert_eq!(
            bincode::deserialize::<Mesh>(&encoded).expect("valid mesh"),
            mesh
        );
        let mut output = Vec::with_capacity(encoded.len());
        group.bench_function("serialize", |b| {
            b.iter(|| {
                output.clear();
                bincode::serialize_into(&mut output, black_box(&mesh)).expect("serializable mesh");
                black_box(&output);
            })
        });
        group.bench_function("deserialize", |b| {
            b.iter(|| {
                let read = bincode::deserialize::<Mesh>(black_box(&encoded)).expect("valid mesh");
                observe(&read);
            })
        });
    }

    #[cfg(feature = "rkyv")]
    {
        use rkyv::{
            Deserialize,
            ser::{
                Serializer,
                serializers::{AlignedSerializer, AllocSerializer},
            },
        };

        let mut group = c.benchmark_group("mesh/rkyv");
        let encoded = rkyv::to_bytes::<_, 1024>(&mesh).expect("serializable mesh");
        let archive = rkyv::check_archived_root::<Mesh>(&encoded).expect("valid archived mesh");
        let decoded: Mesh = archive
            .deserialize(&mut rkyv::Infallible)
            .expect("infallible mesh deserialization");
        assert_eq!(decoded, mesh);

        let mut output = rkyv::AlignedVec::with_capacity(encoded.len());
        group.bench_function("serialize", |b| {
            b.iter(|| {
                output.clear();
                let mut serializer = AllocSerializer::<1024>::new(
                    AlignedSerializer::new(std::mem::take(&mut output)),
                    Default::default(),
                    Default::default(),
                );
                black_box(
                    serializer
                        .serialize_value(black_box(&mesh))
                        .expect("serializable mesh"),
                );
                output = serializer.into_serializer().into_inner();
                black_box(&output);
            })
        });
        group.bench_function("read", |b| {
            b.iter(|| {
                let read = rkyv::check_archived_root::<Mesh>(black_box(&encoded))
                    .expect("valid archived mesh");
                for triangle in read.triangles.iter() {
                    observe_triangle!(triangle);
                }
            })
        });
        group.bench_function("deserialize", |b| {
            b.iter(|| {
                let archive = rkyv::check_archived_root::<Mesh>(black_box(&encoded))
                    .expect("valid archived mesh");
                let read: Mesh = archive
                    .deserialize(&mut rkyv::Infallible)
                    .expect("infallible mesh deserialization");
                observe(&read);
            })
        });
    }

    #[cfg(feature = "speedy")]
    {
        use speedy::{Readable, Writable};

        let mut group = c.benchmark_group("mesh/speedy");
        let encoded = mesh.write_to_vec().expect("serializable mesh");
        assert_eq!(Mesh::read_from_buffer(&encoded).expect("valid mesh"), mesh);
        let mut output = vec![0; encoded.len()];
        group.bench_function("serialize", |b| {
            b.iter(|| {
                black_box(&mesh)
                    .write_to_buffer(&mut output)
                    .expect("serializable mesh");
                black_box(&output);
            })
        });
        group.bench_function("deserialize", |b| {
            b.iter(|| {
                let read = Mesh::read_from_buffer(black_box(&encoded)).expect("valid mesh");
                observe(&read);
            })
        });
    }
}
