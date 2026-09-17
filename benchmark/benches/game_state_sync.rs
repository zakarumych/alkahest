use std::hint::black_box;

use alkahest::{Deserialize, Formula, Lazy, List, Serialize, alkahest};
use criterion::Criterion;

#[cfg(feature = "rkyv")]
use bytecheck::CheckBytes;

#[derive(Debug, PartialEq, Formula, Serialize, Deserialize)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
#[cfg_attr(feature = "rkyv", archive_attr(derive(CheckBytes)))]
#[cfg_attr(feature = "speedy", derive(speedy::Writable, speedy::Readable))]
struct EntityUpdate {
    id: u32,
    position: [f32; 3],
    orientation: [f32; 4],
    velocity: [f32; 3],
    health: u16,
    status: u8,
}

#[derive(Debug, PartialEq, Formula, Serialize, Deserialize)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
#[cfg_attr(feature = "rkyv", archive_attr(derive(CheckBytes)))]
#[cfg_attr(feature = "speedy", derive(speedy::Writable, speedy::Readable))]
struct GameStateSync {
    tick: u32,
    baseline_tick: u32,
    updates: Vec<EntityUpdate>,
    removed: Vec<u32>,
}

#[alkahest(Deserialize<'de, GameStateSync>)]
struct GameStateSyncRead<'de> {
    tick: u32,
    baseline_tick: u32,
    updates: Lazy<'de, List<EntityUpdate>>,
    removed: Lazy<'de, List<u32>>,
}

// A delta snapshot at 60 Hz: 32 changed entities and four despawns.
fn packet() -> GameStateSync {
    GameStateSync {
        tick: 36_000,
        baseline_tick: 35_997,
        updates: (0..32)
            .map(|i| {
                let phase = i as f32 * 0.125;
                let (sin, cos) = (phase * 0.5).sin_cos();
                EntityUpdate {
                    id: 1_000 + i,
                    position: [phase * 4.0 - 8.0, (i % 3) as f32 * 0.5, phase * -2.0],
                    orientation: [0.0, sin, 0.0, cos],
                    velocity: [phase - 2.0, 0.0, 1.0 - phase * 0.25],
                    health: (100 - i * 3) as u16,
                    status: (i % 8) as u8,
                }
            })
            .collect(),
        removed: vec![211, 307, 419, 523],
    }
}

// Touch every update field for both owned and archived packets.
macro_rules! observe_update {
    ($update:expr) => {{
        let update = $update;
        black_box(update.id);
        black_box(update.position);
        black_box(update.orientation);
        black_box(update.velocity);
        black_box(update.health);
        black_box(update.status);
    }};
}

fn observe(packet: &GameStateSync) {
    black_box(packet.tick);
    black_box(packet.baseline_tick);
    for update in &packet.updates {
        observe_update!(update);
    }
    for id in &packet.removed {
        black_box(*id);
    }
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let packet = packet();

    {
        let mut group = c.benchmark_group("game-state-sync/alkahest");
        let mut encoded = Vec::new();
        let size = alkahest::serialize_to_vec::<GameStateSync, _>(&packet, &mut encoded);
        let encoded = &encoded[..size];
        assert_eq!(
            alkahest::deserialize::<GameStateSync, GameStateSync>(encoded)
                .expect("valid game state snapshot"),
            packet
        );
        let read = alkahest::deserialize::<GameStateSync, GameStateSyncRead>(encoded)
            .expect("valid lazy game state snapshot");
        assert_eq!(read.tick, packet.tick);
        assert_eq!(read.baseline_tick, packet.baseline_tick);
        assert_eq!(
            read.updates
                .iter::<EntityUpdate>()
                .expect("valid update list")
                .collect::<Result<Vec<_>, _>>()
                .expect("valid updates"),
            packet.updates
        );
        assert_eq!(
            read.removed
                .iter::<u32>()
                .expect("valid removal list")
                .collect::<Result<Vec<_>, _>>()
                .expect("valid removals"),
            packet.removed
        );

        let mut output = Vec::with_capacity(size);
        group.bench_function("serialize", |b| {
            b.iter(|| {
                black_box(alkahest::serialize_to_vec::<GameStateSync, _>(
                    black_box(&packet),
                    &mut output,
                ));
                black_box(&output);
            })
        });
        group.bench_function("read", |b| {
            b.iter(|| {
                let read =
                    alkahest::deserialize::<GameStateSync, GameStateSyncRead>(black_box(encoded))
                        .expect("valid lazy game state snapshot");
                black_box(read.tick);
                black_box(read.baseline_tick);
                for update in read
                    .updates
                    .iter::<EntityUpdate>()
                    .expect("valid update list")
                {
                    observe_update!(update.expect("valid update"));
                }
                for id in read.removed.iter::<u32>().expect("valid removal list") {
                    black_box(id.expect("valid removal"));
                }
            })
        });
        group.bench_function("deserialize", |b| {
            b.iter(|| {
                let read =
                    alkahest::deserialize::<GameStateSync, GameStateSync>(black_box(encoded))
                        .expect("valid game state snapshot");
                observe(&read);
            })
        });
    }

    #[cfg(feature = "bincode")]
    {
        let mut group = c.benchmark_group("game-state-sync/bincode");
        let encoded = bincode::serialize(&packet).expect("serializable game state snapshot");
        assert_eq!(
            bincode::deserialize::<GameStateSync>(&encoded).expect("valid game state snapshot"),
            packet
        );
        let mut output = Vec::with_capacity(encoded.len());
        group.bench_function("serialize", |b| {
            b.iter(|| {
                output.clear();
                bincode::serialize_into(&mut output, black_box(&packet))
                    .expect("serializable game state snapshot");
                black_box(&output);
            })
        });
        group.bench_function("deserialize", |b| {
            b.iter(|| {
                let read = bincode::deserialize::<GameStateSync>(black_box(&encoded))
                    .expect("valid game state snapshot");
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

        let mut group = c.benchmark_group("game-state-sync/rkyv");
        let encoded = rkyv::to_bytes::<_, 1024>(&packet).expect("serializable game state snapshot");
        let archive = rkyv::check_archived_root::<GameStateSync>(&encoded)
            .expect("valid archived game state snapshot");
        let decoded: GameStateSync = archive
            .deserialize(&mut rkyv::Infallible)
            .expect("infallible game state deserialization");
        assert_eq!(decoded, packet);

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
                        .serialize_value(black_box(&packet))
                        .expect("serializable game state snapshot"),
                );
                output = serializer.into_serializer().into_inner();
                black_box(&output);
            })
        });
        group.bench_function("read", |b| {
            b.iter(|| {
                let read = rkyv::check_archived_root::<GameStateSync>(black_box(&encoded))
                    .expect("valid archived game state snapshot");
                black_box(read.tick);
                black_box(read.baseline_tick);
                for update in read.updates.iter() {
                    observe_update!(update);
                }
                for id in read.removed.iter() {
                    black_box(*id);
                }
            })
        });
        group.bench_function("deserialize", |b| {
            b.iter(|| {
                let archive = rkyv::check_archived_root::<GameStateSync>(black_box(&encoded))
                    .expect("valid archived game state snapshot");
                let read: GameStateSync = archive
                    .deserialize(&mut rkyv::Infallible)
                    .expect("infallible game state deserialization");
                observe(&read);
            })
        });
    }

    #[cfg(feature = "speedy")]
    {
        use speedy::{Readable, Writable};

        let mut group = c.benchmark_group("game-state-sync/speedy");
        let encoded = packet
            .write_to_vec()
            .expect("serializable game state snapshot");
        assert_eq!(
            GameStateSync::read_from_buffer(&encoded).expect("valid game state snapshot"),
            packet
        );
        let mut output = vec![0; encoded.len()];
        group.bench_function("serialize", |b| {
            b.iter(|| {
                black_box(&packet)
                    .write_to_buffer(&mut output)
                    .expect("serializable game state snapshot");
                black_box(&output);
            })
        });
        group.bench_function("deserialize", |b| {
            b.iter(|| {
                let read = GameStateSync::read_from_buffer(black_box(&encoded))
                    .expect("valid game state snapshot");
                observe(&read);
            })
        });
    }
}
