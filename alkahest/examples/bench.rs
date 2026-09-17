use std::hint::black_box;

use alkahest::{Deserialize, Element, Formula, Lazy, List, Serialize, alkahest};

use rand::{
    Rng, RngExt, SeedableRng,
    distr::{Alphanumeric, SampleString},
    rngs::SmallRng,
};

#[derive(Debug, Clone, PartialEq, Formula, Serialize, Deserialize)]
pub enum GameMessage {
    Client(ClientMessage),
    Server(ServerMessage),
}

#[derive(Debug, PartialEq)]
#[alkahest(Deserialize<'de, GameMessage>)]
pub enum GameMessageRead<'de> {
    Client(ClientMessageRead<'de>),
    Server(ServerMessageRead<'de>),
}

#[derive(Debug, Clone, PartialEq, Formula, Serialize, Deserialize)]
pub enum ClientMessage {
    ClientData { nickname: String, clan: String },
    Chat(String),
}

#[derive(Debug, PartialEq)]
#[alkahest(Deserialize<'de, ClientMessage>)]
pub enum ClientMessageRead<'de> {
    ClientData { nickname: &'de str, clan: &'de str },
    Chat(&'de str),
}

#[derive(Debug, Clone, PartialEq, Formula, Serialize, Deserialize)]
pub enum ServerMessage {
    ServerData(u64),
    ClientChat { client_id: u64, message: String },
}

#[derive(Debug, PartialEq)]
#[alkahest(Deserialize<'de, ServerMessage>)]
pub enum ServerMessageRead<'de> {
    ServerData(u64),
    ClientChat { client_id: u64, message: &'de str },
}

#[derive(Debug, PartialEq)]
#[alkahest(Formula where G: Element)]
#[alkahest(for<X> Serialize<NetPacket<X>> where X: Element, G: Serialize<X::Formula>)]
#[alkahest(for<X> Deserialize<NetPacket<X>> where X: Element, G: Deserialize<'de, X::Formula>)]
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

fn main() {
    const LEN: usize = 5;
    let packet = NetPacket {
        game_messages: messages(SmallRng::seed_from_u64(42), LEN).collect(),
    };
    let expected_read: Vec<_> = packet.game_messages.iter().map(as_read).collect();

    {
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
        {
            {
                black_box(alkahest::serialize_to_vec::<NetPacket<GameMessage>, _>(
                    black_box(&packet),
                    &mut output,
                ));
                black_box(&output);
            }
        }
        {
            {
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
            }
        }
        {
            {
                let read = alkahest::deserialize::<NetPacket<GameMessage>, NetPacket<GameMessage>>(
                    black_box(encoded),
                )
                .expect("valid network packet");
                observe(&read);
            }
        }
    }
}
