use alloc::{string::String, vec::Vec};

use rand::{
    distributions::{Alphanumeric, DistString},
    Rng, SeedableRng,
};

use crate::{
    alkahest, read_packet, write_packet_to_vec, Formula, Lazy, SerIter, Serialize, SerializeRef,
};

#[derive(Debug, Clone, PartialEq, Eq)]
#[alkahest(Formula, Serialize, Deserialize)]
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

#[derive(Debug, PartialEq, Eq, Clone)]
#[alkahest(Formula, Serialize, Deserialize)]
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

#[derive(Debug, PartialEq, Eq, Clone)]
#[alkahest(Formula, Serialize, Deserialize)]
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
#[alkahest(Formula, Serialize, Deserialize)]
pub struct NetPacket<G> {
    pub game_messages: Vec<G>,
}

#[derive(Debug)]
#[alkahest(for<X: Formula> Serialize<NetPacket<X>> where G: Serialize<[X]>)]
#[alkahest(for<X: Formula> SerializeRef<NetPacket<X>> where G: SerializeRef<[X]>)]
pub struct NetPacketWrite<G> {
    pub game_messages: G,
}

#[derive(Debug)]
#[alkahest(Deserialize<'de, NetPacket::<G>> where G: Formula)]
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
        3 => GameMessage::Server(ServerMessage::ServerData(rng.gen())),
        _ => unreachable!(),
    })
    .take(len)
}

#[cfg(all(feature = "alloc", feature = "derive"))]
#[test]
fn test_net_packet() {
    let rng = rand::rngs::SmallRng::from_rng(rand::thread_rng()).unwrap();

    #[cfg(feature = "fixed8")]
    const LEN: usize = 1;

    #[cfg(not(feature = "fixed8"))]
    const LEN: usize = 1000;

    let mut buffer = Vec::new();
    let size = write_packet_to_vec::<NetPacket<GameMessage>, _>(
        NetPacketWrite {
            game_messages: SerIter(messages(rng.clone(), LEN)),
        },
        &mut buffer,
    );

    let mut buffer2 = Vec::new();
    let size2 = write_packet_to_vec::<NetPacket<GameMessage>, _>(
        NetPacket {
            game_messages: messages(rng, LEN).collect::<Vec<_>>(),
        },
        &mut buffer2,
    );

    assert_eq!(size, size2);
    assert_eq!(buffer[..size], buffer2[..size]);

    let (packet, _) =
        read_packet::<NetPacket<GameMessage>, NetPacketRead<GameMessage>>(&buffer[..]).unwrap();

    for message in packet.game_messages.iter::<GameMessageRead>() {
        match message.unwrap() {
            GameMessageRead::Client(ClientMessageRead::ClientData { nickname, clan }) => {
                drop(nickname);
                drop(clan);
            }
            GameMessageRead::Client(ClientMessageRead::Chat(message)) => {
                drop(message);
            }
            GameMessageRead::Server(ServerMessageRead::ServerData(data)) => {
                drop(data);
            }
            GameMessageRead::Server(ServerMessageRead::ClientChat { client_id, message }) => {
                drop(client_id);
                drop(message);
            }
        }
    }
}
