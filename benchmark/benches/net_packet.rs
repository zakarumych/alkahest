#[cfg(feature = "rkyv")]
extern crate bytecheck;

#[cfg(feature = "rkyv")]
extern crate rkyv;

#[cfg(feature = "speedy")]
extern crate speedy;

use std::hint::black_box;

use alkahest::{Deserialize, Element, Formula, Lazy, List, Serialize, alkahest};
use criterion::Criterion;

#[cfg(feature = "rkyv")]
use bytecheck::CheckBytes;
use rand::{
    Rng, RngExt, SeedableRng,
    distr::{Alphanumeric, SampleString},
    rngs::SmallRng,
};

#[derive(Debug, Clone, PartialEq, Formula, Serialize, Deserialize)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
#[cfg_attr(feature = "rkyv", archive_attr(derive(CheckBytes)))]
#[cfg_attr(feature = "speedy", derive(speedy::Writable, speedy::Readable))]
pub enum GameMessage {
    Client(ClientMessage),
    Server(ServerMessage),
}

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "speedy", derive(speedy::Readable))]
#[alkahest(Deserialize<'de, GameMessage>)]
pub enum GameMessageRead<'de> {
    Client(ClientMessageRead<'de>),
    Server(ServerMessageRead<'de>),
}

#[derive(Debug, Clone, PartialEq, Formula, Serialize, Deserialize)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
#[cfg_attr(feature = "rkyv", archive_attr(derive(CheckBytes)))]
#[cfg_attr(feature = "speedy", derive(speedy::Writable, speedy::Readable))]
pub enum ClientMessage {
    ClientData { nickname: String, clan: String },
    Chat(String),
}

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "speedy", derive(speedy::Readable))]
#[alkahest(Deserialize<'de, ClientMessage>)]
pub enum ClientMessageRead<'de> {
    ClientData { nickname: &'de str, clan: &'de str },
    Chat(&'de str),
}

#[derive(Debug, Clone, PartialEq, Formula, Serialize, Deserialize)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
#[cfg_attr(feature = "rkyv", archive_attr(derive(CheckBytes)))]
#[cfg_attr(feature = "speedy", derive(speedy::Writable, speedy::Readable))]
pub enum ServerMessage {
    ServerData(u64),
    ClientChat { client_id: u64, message: String },
}

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "speedy", derive(speedy::Readable))]
#[alkahest(Deserialize<'de, ServerMessage>)]
pub enum ServerMessageRead<'de> {
    ServerData(u64),
    ClientChat { client_id: u64, message: &'de str },
}

#[derive(Debug, PartialEq)]
#[alkahest(Formula where G: Element)]
#[alkahest(for<X> Serialize<NetPacket<X>> where X: Element, G: Serialize<X::Formula>)]
#[alkahest(for<X> Deserialize<NetPacket<X>> where X: Element, G: Deserialize<'de, X::Formula>)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
#[cfg_attr(feature = "rkyv", archive_attr(derive(CheckBytes)))]
#[cfg_attr(feature = "speedy", derive(speedy::Writable, speedy::Readable))]
pub struct NetPacket<G> {
    pub game_messages: Vec<G>,
}

#[alkahest(Deserialize<'de, NetPacket<G>> where G: Element)]
pub struct NetPacketRead<'de, G> {
    pub game_messages: Lazy<'de, List<G>>,
}

fn get_string(rng: &mut impl Rng) -> String {
    Alphanumeric.sample_string(rng, 8)
}

fn messages<'a>(mut rng: impl Rng + 'a, len: usize) -> impl Iterator<Item = GameMessage> + 'a {
    core::iter::repeat_with(move || match rng.random_range(0..4) {
        0 => GameMessage::Client(ClientMessage::ClientData {
            nickname: get_string(&mut rng),
            clan: get_string(&mut rng),
        }),
        1 => GameMessage::Client(ClientMessage::Chat(get_string(&mut rng))),
        2 => GameMessage::Server(ServerMessage::ClientChat {
            client_id: rng.random(),
            message: get_string(&mut rng),
        }),
        3 => GameMessage::Server(ServerMessage::ServerData(rng.random_range(0..10))),
        _ => unreachable!(),
    })
    .take(len)
    // Make size unpredictable for `FromIterator` as this is common in real-world.
    .filter(|msg| !matches!(msg, GameMessage::Server(ServerMessage::ServerData(3..=10))))
}

fn as_read(message: &GameMessage) -> GameMessageRead<'_> {
    match message {
        GameMessage::Client(ClientMessage::ClientData { nickname, clan }) => {
            GameMessageRead::Client(ClientMessageRead::ClientData { nickname, clan })
        }
        GameMessage::Client(ClientMessage::Chat(message)) => {
            GameMessageRead::Client(ClientMessageRead::Chat(message))
        }
        GameMessage::Server(ServerMessage::ServerData(data)) => {
            GameMessageRead::Server(ServerMessageRead::ServerData(*data))
        }
        GameMessage::Server(ServerMessage::ClientChat { client_id, message }) => {
            GameMessageRead::Server(ServerMessageRead::ClientChat {
                client_id: *client_id,
                message,
            })
        }
    }
}

fn observe_message(message: GameMessageRead<'_>) {
    match message {
        GameMessageRead::Client(ClientMessageRead::ClientData { nickname, clan }) => {
            black_box(nickname);
            black_box(clan);
        }
        GameMessageRead::Client(ClientMessageRead::Chat(message)) => {
            black_box(message);
        }
        GameMessageRead::Server(ServerMessageRead::ServerData(data)) => {
            black_box(data);
        }
        GameMessageRead::Server(ServerMessageRead::ClientChat { client_id, message }) => {
            black_box(client_id);
            black_box(message);
        }
    }
}

fn observe(packet: &NetPacket<GameMessage>) {
    for message in &packet.game_messages {
        observe_message(as_read(message));
    }
}

#[cfg(feature = "rkyv")]
fn archived_as_read(message: &ArchivedGameMessage) -> GameMessageRead<'_> {
    match message {
        ArchivedGameMessage::Client(ArchivedClientMessage::ClientData { nickname, clan }) => {
            GameMessageRead::Client(ClientMessageRead::ClientData {
                nickname: nickname.as_str(),
                clan: clan.as_str(),
            })
        }
        ArchivedGameMessage::Client(ArchivedClientMessage::Chat(message)) => {
            GameMessageRead::Client(ClientMessageRead::Chat(message.as_str()))
        }
        ArchivedGameMessage::Server(ArchivedServerMessage::ServerData(data)) => {
            GameMessageRead::Server(ServerMessageRead::ServerData(*data))
        }
        ArchivedGameMessage::Server(ArchivedServerMessage::ClientChat { client_id, message }) => {
            GameMessageRead::Server(ServerMessageRead::ClientChat {
                client_id: *client_id,
                message: message.as_str(),
            })
        }
    }
}

pub fn criterion_benchmark(c: &mut Criterion) {
    const LEN: usize = 100;
    let packet = NetPacket {
        game_messages: messages(SmallRng::seed_from_u64(42), LEN).collect(),
    };
    let expected_read: Vec<_> = packet.game_messages.iter().map(as_read).collect();

    {
        let mut group = c.benchmark_group("net-packet/alkahest");
        let mut encoded = Vec::new();
        let size = alkahest::serialize_to_vec::<NetPacket<GameMessage>, _>(&packet, &mut encoded);
        let encoded = &encoded[..size];
        assert_eq!(
            alkahest::deserialize::<NetPacket<GameMessage>, NetPacket<GameMessage>>(encoded)
                .expect("valid network packet"),
            packet
        );
        let read =
            alkahest::deserialize::<NetPacket<GameMessage>, NetPacketRead<GameMessage>>(encoded)
                .expect("valid lazy network packet");
        assert_eq!(
            read.game_messages
                .iter::<GameMessageRead>()
                .expect("valid message list")
                .collect::<Result<Vec<_>, _>>()
                .expect("valid messages"),
            expected_read
        );

        let mut output = Vec::with_capacity(size);
        group.bench_function("serialize", |b| {
            b.iter(|| {
                black_box(alkahest::serialize_to_vec::<NetPacket<GameMessage>, _>(
                    black_box(&packet),
                    &mut output,
                ));
                black_box(&output);
            })
        });
        group.bench_function("read", |b| {
            b.iter(|| {
                let read = alkahest::deserialize::<
                    NetPacket<GameMessage>,
                    NetPacketRead<GameMessage>,
                >(black_box(encoded))
                .expect("valid lazy network packet");
                for message in read
                    .game_messages
                    .iter::<GameMessageRead>()
                    .expect("valid message list")
                {
                    observe_message(message.expect("valid message"));
                }
            })
        });
        group.bench_function("deserialize", |b| {
            b.iter(|| {
                let read = alkahest::deserialize::<NetPacket<GameMessage>, NetPacket<GameMessage>>(
                    black_box(encoded),
                )
                .expect("valid network packet");
                observe(&read);
            })
        });
    }

    #[cfg(feature = "bincode")]
    {
        let mut group = c.benchmark_group("net-packet/bincode");
        let encoded = bincode::serialize(&packet).expect("serializable network packet");
        assert_eq!(
            bincode::deserialize::<NetPacket<GameMessage>>(&encoded).expect("valid network packet"),
            packet
        );
        let mut output = Vec::with_capacity(encoded.len());
        group.bench_function("serialize", |b| {
            b.iter(|| {
                output.clear();
                bincode::serialize_into(&mut output, black_box(&packet))
                    .expect("serializable network packet");
                black_box(&output);
            })
        });
        group.bench_function("deserialize", |b| {
            b.iter(|| {
                let read = bincode::deserialize::<NetPacket<GameMessage>>(black_box(&encoded))
                    .expect("valid network packet");
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

        let mut group = c.benchmark_group("net-packet/rkyv");
        let encoded = rkyv::to_bytes::<_, 16536>(&packet).expect("serializable network packet");
        let archive = rkyv::check_archived_root::<NetPacket<GameMessage>>(&encoded)
            .expect("valid archived network packet");
        let decoded: NetPacket<GameMessage> = archive
            .deserialize(&mut rkyv::Infallible)
            .expect("infallible network packet deserialization");
        assert_eq!(decoded, packet);
        assert_eq!(
            archive
                .game_messages
                .iter()
                .map(archived_as_read)
                .collect::<Vec<_>>(),
            expected_read
        );

        let mut output = rkyv::AlignedVec::with_capacity(encoded.len());
        group.bench_function("serialize", |b| {
            b.iter(|| {
                output.clear();
                let mut serializer = AllocSerializer::<16536>::new(
                    AlignedSerializer::new(std::mem::take(&mut output)),
                    Default::default(),
                    Default::default(),
                );
                black_box(
                    serializer
                        .serialize_value(black_box(&packet))
                        .expect("serializable network packet"),
                );
                output = serializer.into_serializer().into_inner();
                black_box(&output);
            })
        });
        group.bench_function("read", |b| {
            b.iter(|| {
                let read = rkyv::check_archived_root::<NetPacket<GameMessage>>(black_box(&encoded))
                    .expect("valid archived network packet");
                for message in read.game_messages.iter() {
                    observe_message(archived_as_read(message));
                }
            })
        });
        group.bench_function("deserialize", |b| {
            b.iter(|| {
                let archive =
                    rkyv::check_archived_root::<NetPacket<GameMessage>>(black_box(&encoded))
                        .expect("valid archived network packet");
                let read: NetPacket<GameMessage> = archive
                    .deserialize(&mut rkyv::Infallible)
                    .expect("infallible network packet deserialization");
                observe(&read);
            })
        });
    }

    #[cfg(feature = "speedy")]
    {
        use speedy::{Readable, Writable};

        let mut group = c.benchmark_group("net-packet/speedy");
        let encoded = packet.write_to_vec().expect("serializable network packet");
        assert_eq!(
            NetPacket::<GameMessage>::read_from_buffer(&encoded).expect("valid network packet"),
            packet
        );
        assert_eq!(
            NetPacket::<GameMessageRead>::read_from_buffer(&encoded)
                .expect("valid borrowed network packet")
                .game_messages,
            expected_read
        );
        let mut output = vec![0; encoded.len()];
        group.bench_function("serialize", |b| {
            b.iter(|| {
                black_box(&packet)
                    .write_to_buffer(&mut output)
                    .expect("serializable network packet");
                black_box(&output);
            })
        });
        group.bench_function("read", |b| {
            b.iter(|| {
                let read = NetPacket::<GameMessageRead>::read_from_buffer(black_box(&encoded))
                    .expect("valid borrowed network packet");
                for message in read.game_messages {
                    observe_message(message);
                }
            })
        });
        group.bench_function("deserialize", |b| {
            b.iter(|| {
                let read = NetPacket::<GameMessage>::read_from_buffer(black_box(&encoded))
                    .expect("valid network packet");
                observe(&read);
            })
        });
    }
}
