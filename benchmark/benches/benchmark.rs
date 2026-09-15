extern crate alkahest;
extern crate criterion;
extern crate rand;

mod game_state_sync;
mod net_packet;

#[cfg(feature = "rkyv")]
extern crate bytecheck;

#[cfg(feature = "rkyv")]
extern crate rkyv;

#[cfg(feature = "speedy")]
extern crate speedy;

use criterion::{criterion_group, criterion_main};

criterion_group!(
    benches,
    net_packet::criterion_benchmark,
    game_state_sync::criterion_benchmark
);
criterion_main!(benches);
