use std::hint::black_box;

use alkahest::{Deserialize, Element, Formula, Lazy, List, MakeIter, Serialize, alkahest};

use rand::{
    Rng, RngExt, SeedableRng,
    distr::{Alphanumeric, SampleString},
    rngs::SmallRng,
};

#[derive(Debug, Clone, Formula, Serialize, Deserialize)]
pub enum GameMessage {
    Client(ClientMessage),
    Server(ServerMessage),
}

#[derive(Debug)]
#[alkahest(Deserialize<'de, GameMessage>)]
pub enum GameMessageRead<'de> {
    Client(ClientMessageRead<'de>),
    Server(ServerMessageRead<'de>),
}

#[derive(Debug, Clone, Formula, Serialize, Deserialize)]
pub enum ClientMessage {
    ClientData { nickname: String, clan: String },
    Chat(String),
}

#[derive(Debug)]
#[alkahest(Deserialize<'de, ClientMessage>)]
pub enum ClientMessageRead<'de> {
    ClientData { nickname: &'de str, clan: &'de str },
    Chat(&'de str),
}

#[derive(Debug, Clone, Formula, Serialize, Deserialize)]
pub enum ServerMessage {
    ServerData(u64),
    ClientChat { client_id: u64, message: String },
}

#[derive(Debug)]
#[alkahest(Deserialize<'de, ServerMessage>)]
pub enum ServerMessageRead<'de> {
    ServerData(u64),
    ClientChat { client_id: u64, message: &'de str },
}

#[derive(Debug)]
#[alkahest(Formula where G: Element)]
#[alkahest(for<X> Serialize<NetPacket<X>> where X: Element, G: Serialize<X::Formula>)]
#[alkahest(for<X> Deserialize<NetPacket<X>> where X: Element, G: Deserialize<'de, X::Formula>)]
pub struct NetPacket<G> {
    pub game_messages: Vec<G>,
}

#[derive(Debug)]
#[alkahest(for<X: Element> Serialize<NetPacket<X>> where G: Serialize<List<X>>)]
pub struct NetPacketWrite<G> {
    pub game_messages: G,
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

fn main() {
    let mut buffer = Vec::with_capacity(1 << 14);
    buffer.resize(buffer.capacity(), 0);
    let rng = SmallRng::seed_from_u64(42);

    const LEN: usize = 4;
    let mut size = 0;

    {
        {
            {
                size = alkahest::serialize_to_vec::<NetPacket<GameMessage>, _>(
                    &NetPacketWrite {
                        game_messages: MakeIter(|| messages(rng.clone(), black_box(LEN))),
                    },
                    &mut buffer,
                );
            }
        }

        {
            {
                let packet = alkahest::deserialize::<
                    NetPacket<GameMessage>,
                    NetPacketRead<GameMessage>,
                >(&buffer[..size])
                .unwrap();

                for message in packet.game_messages.iter::<GameMessageRead>().unwrap() {
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
            }
        }

        {
            {
                let packet =
                    alkahest::deserialize::<NetPacket<GameMessage>, NetPacket<GameMessage>>(
                        &buffer[..size],
                    )
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
            }
        }
    }
}
