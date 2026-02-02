use std::{hint::black_box, mem::size_of};

use alkahest::*;
use rand::{distributions::Standard, prelude::Distribution, Rng};

#[derive(Clone, Copy)]
#[alkahest(Formula, Serialize, SerializeRef, Deserialize)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Distribution<Vector3> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Vector3 {
        Vector3 {
            x: rng.gen(),
            y: rng.gen(),
            z: rng.gen(),
        }
    }
}

#[derive(Clone, Copy)]
#[alkahest(Formula, Serialize, SerializeRef, Deserialize)]
pub struct Triangle {
    pub v0: Vector3,
    pub v1: Vector3,
    pub v2: Vector3,
    pub normal: Vector3,
}

impl Distribution<Triangle> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Triangle {
        let v0 = rng.gen();
        let v1 = rng.gen();
        let v2 = rng.gen();
        let normal = rng.gen();

        Triangle { v0, v1, v2, normal }
    }
}

#[alkahest(Formula)]
pub struct MeshFormula {
    pub triangles: [Triangle],
}

#[derive(Clone)]
#[alkahest(Serialize<MeshFormula>, SerializeRef<MeshFormula>, Deserialize<'_, MeshFormula>)]
pub struct Mesh {
    pub triangles: Vec<Triangle>,
}

#[alkahest(Deserialize<'a, MeshFormula>)]
pub struct LazyMesh<'a> {
    pub triangles: Lazy<'a, [Triangle]>,
}

#[inline]
fn do_serialize(mesh: &Mesh, buffer: &mut [u8]) -> usize {
    alkahest::write_packet_unchecked::<MeshFormula, _>(&mesh, buffer)
}

fn main() {
    const TRIG_COUNT: usize = 100_000;

    let mesh = Mesh {
        triangles: rand::thread_rng()
            .sample_iter(Standard)
            .take(TRIG_COUNT)
            .collect(),
    };

    let mut mesh = black_box(mesh);

    let mut buffer = Vec::new();
    buffer.resize(TRIG_COUNT * size_of::<Triangle>() + 32, 0);

    for _ in 0..10_000 {
        let size = do_serialize(&mesh, &mut buffer);
        black_box(&buffer[..size]);
        mesh = black_box(mesh);
    }
}
