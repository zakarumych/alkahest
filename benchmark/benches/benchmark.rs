extern crate alkahest;
extern crate criterion;
extern crate rand;

#[cfg(feature = "rkyv")]
extern crate bytecheck;

#[cfg(feature = "rkyv")]
extern crate rkyv;

#[cfg(feature = "speedy")]
extern crate speedy;

use alkahest::{Deserialize, Formula, Lazy, Ref, SerIter, Serialize};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

#[cfg(feature = "rkyv")]
use bytecheck::CheckBytes;
use rand::{
    distributions::{Alphanumeric, DistString},
    rngs::SmallRng,
    thread_rng, Rng, SeedableRng,
};

#[derive(Debug, Clone, Formula, Serialize, Deserialize)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize))]
#[cfg_attr(feature = "rkyv", archive_attr(derive(CheckBytes)))]
#[cfg_attr(feature = "speedy", derive(speedy::Writable, speedy::Readable))]
pub enum GameMessage {
    Client(ClientMessage),
    Server(ServerMessage),
}

#[derive(Debug, Deserialize)]
#[alkahest(GameMessage)]
pub enum GameMessageRead<'de> {
    Client(ClientMessageRead<'de>),
    Server(ServerMessageRead<'de>),
}

#[derive(Debug, Clone, Formula, Serialize, Deserialize)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize))]
#[cfg_attr(feature = "rkyv", archive_attr(derive(CheckBytes)))]
#[cfg_attr(feature = "speedy", derive(speedy::Writable, speedy::Readable))]
pub enum ClientMessage {
    ClientData { nickname: String, clan: String },
    Chat(String),
}

#[derive(Debug, Deserialize)]
#[alkahest(ClientMessage)]
pub enum ClientMessageRead<'de> {
    ClientData { nickname: &'de str, clan: &'de str },
    Chat(&'de str),
}

#[derive(Debug, Clone, Formula, Serialize, Deserialize)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize))]
#[cfg_attr(feature = "rkyv", archive_attr(derive(CheckBytes)))]
#[cfg_attr(feature = "speedy", derive(speedy::Writable, speedy::Readable))]
pub enum ServerMessage {
    ServerData(u64),
    ClientChat { client_id: u64, message: String },
}

#[derive(Debug, Deserialize)]
#[alkahest(ServerMessage)]
pub enum ServerMessageRead<'de> {
    ServerData(u64),
    ClientChat { client_id: u64, message: &'de str },
}

#[derive(Debug, Formula, Serialize, Deserialize)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize))]
#[cfg_attr(feature = "rkyv", archive_attr(derive(CheckBytes)))]
#[cfg_attr(feature = "speedy", derive(speedy::Writable, speedy::Readable))]
pub struct NetPacket<G> {
    pub game_messages: Vec<G>,
}

#[derive(Debug, Serialize)]
#[alkahest(owned(for<X: Formula> NetPacket<X> where G: Serialize<[X]>))]
pub struct NetPacketWrite<G> {
    pub game_messages: G,
}

#[derive(Debug, Deserialize)]
#[alkahest(NetPacket<G> where G: Formula)]
pub struct NetPacketRead<'de, G> {
    pub game_messages: Lazy<'de, [G]>,
}

fn get_string(rng: &mut impl Rng) -> String {
    Alphanumeric.sample_string(rng, 8)
}

fn messages<'a>(mut rng: impl Rng + 'a, len: usize) -> impl Iterator<Item = GameMessage> + 'a {
    core::iter::repeat_with(move || match rng.gen_range(0..4) {
        0 => GameMessage::Client(ClientMessage::ClientData {
            nickname: get_string(&mut rng),
            clan: get_string(&mut rng),
        }),
        1 => GameMessage::Client(ClientMessage::Chat(get_string(&mut rng))),
        2 => GameMessage::Server(ServerMessage::ClientChat {
            client_id: rng.gen(),
            message: get_string(&mut rng),
        }),
        3 => GameMessage::Server(ServerMessage::ServerData(rng.gen_range(0..10))),
        _ => unreachable!(),
    })
    .take(len)
    // Make size unpredictable for `FromIterator` as this is common in real-world.
    .filter(|msg| !matches!(msg, GameMessage::Server(ServerMessage::ServerData(3..=10))))
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut buffer = Vec::with_capacity(1 << 14);
    buffer.resize(buffer.capacity(), 0);
    let mut rng = SmallRng::seed_from_u64(42);

    const LEN: usize = 200;

    {
        let mut group = c.benchmark_group("net-packet/alkahest");
        group.bench_function("serialize", |b| {
            b.iter(|| {
                alkahest::serialize_to_vec::<NetPacket<GameMessage>, _>(
                    NetPacketWrite {
                        game_messages: SerIter(messages(rng.clone(), black_box(LEN))),
                    },
                    &mut buffer,
                );
            })
        });

        group.bench_function("read", |b| {
            b.iter(|| {
                let (packet, _) = alkahest::deserialize::<
                    NetPacket<GameMessage>,
                    NetPacketRead<GameMessage>,
                >(&buffer[..])
                .unwrap();

                for message in packet.game_messages.iter::<GameMessageRead>() {
                    match message.unwrap() {
                        GameMessageRead::Client(ClientMessageRead::ClientData {
                            nickname,
                            clan,
                        }) => {
                            black_box(nickname);
                            black_box(clan);
                        }
                        GameMessageRead::Client(ClientMessageRead::Chat(message)) => {
                            black_box(message);
                        }
                        GameMessageRead::Server(ServerMessageRead::ServerData(data)) => {
                            black_box(data);
                        }
                        GameMessageRead::Server(ServerMessageRead::ClientChat {
                            client_id,
                            message,
                        }) => {
                            black_box(client_id);
                            black_box(message);
                        }
                    }
                }
            })
        });
    }

    #[cfg(feature = "bincode")]
    {
        let mut group = c.benchmark_group("net-packet/bincode");
        group.bench_function("serialize", |b| {
            b.iter(|| {
                buffer.clear();
                bincode::serialize_into(
                    &mut buffer,
                    &NetPacket {
                        game_messages: messages(rng.clone(), black_box(LEN)).collect(),
                    },
                )
                .unwrap();
            })
        });

        group.bench_function("read", |b| {
            b.iter(|| {
                let packet = bincode::deserialize::<NetPacket<GameMessage>>(&buffer).unwrap();

                for message in packet.game_messages.iter() {
                    match message {
                        GameMessage::Client(ClientMessage::ClientData { nickname, clan }) => {
                            black_box(nickname);
                            black_box(clan);
                        }
                        GameMessage::Client(ClientMessage::Chat(message)) => {
                            black_box(message);
                        }
                        GameMessage::Server(ServerMessage::ServerData(data)) => {
                            black_box(data);
                        }
                        GameMessage::Server(ServerMessage::ClientChat { client_id, message }) => {
                            black_box(client_id);
                            black_box(message);
                        }
                    }
                }
            })
        });
    }

    #[cfg(feature = "rkyv")]
    {
        let mut group = c.benchmark_group("net-packet/rkyv");
        let mut rkyv_ser = rkyv::ser::serializers::AllocSerializer::<1024>::default();

        group.bench_function("serialize", |b| {
            b.iter(|| {
                use rkyv::ser::Serializer;

                rkyv_ser
                    .serialize_value(&NetPacket {
                        game_messages: messages(rng.clone(), black_box(LEN)).collect(),
                    })
                    .unwrap()
            })
        });

        let vec: rkyv::AlignedVec = rkyv_ser.into_serializer().into_inner();
        group.bench_function("read", |b| {
            b.iter(|| {
                let packet = rkyv::check_archived_root::<NetPacket<GameMessage>>(&vec[..]).unwrap();

                for message in packet.game_messages.iter() {
                    match message {
                        ArchivedGameMessage::Client(ArchivedClientMessage::ClientData {
                            nickname,
                            clan,
                        }) => {
                            black_box(nickname);
                            black_box(clan);
                        }
                        ArchivedGameMessage::Client(ArchivedClientMessage::Chat(message)) => {
                            black_box(message);
                        }
                        ArchivedGameMessage::Server(ArchivedServerMessage::ServerData(data)) => {
                            black_box(data);
                        }
                        ArchivedGameMessage::Server(ArchivedServerMessage::ClientChat {
                            client_id,
                            message,
                        }) => {
                            black_box(client_id);
                            black_box(message);
                        }
                    }
                }
            })
        });
    }

    #[cfg(feature = "speedy")]
    {
        let mut group = c.benchmark_group("net-packet/speedy");

        buffer.clear();
        buffer.resize(buffer.capacity(), 0);

        group.bench_function("serialize", |b| {
            b.iter(|| {
                speedy::Writable::write_to_buffer(
                    &NetPacket {
                        game_messages: messages(rng.clone(), black_box(LEN)).collect(),
                    },
                    &mut buffer,
                )
                .unwrap();
            })
        });

        group.bench_function("read", |b| {
            b.iter(|| {
                let packet =
                    <NetPacket<GameMessage> as speedy::Readable<_>>::read_from_buffer(&buffer)
                        .unwrap();

                for message in packet.game_messages.iter() {
                    match message {
                        GameMessage::Client(ClientMessage::ClientData { nickname, clan }) => {
                            black_box(nickname);
                            black_box(clan);
                        }
                        GameMessage::Client(ClientMessage::Chat(message)) => {
                            black_box(message);
                        }
                        GameMessage::Server(ServerMessage::ServerData(data)) => {
                            black_box(data);
                        }
                        GameMessage::Server(ServerMessage::ClientChat { client_id, message }) => {
                            black_box(client_id);
                            black_box(message);
                        }
                    }
                }
            })
        });
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
