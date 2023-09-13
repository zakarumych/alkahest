use alloc::{string::String, vec::Vec};

use rand::{
    distributions::{Alphanumeric, DistString},
    Rng, SeedableRng,
};

use crate::{
    alkahest, read_packet, write_packet_to_vec, Deserialize, Formula, Lazy, SerIter, Serialize, SerializeRef, Ref,
};

#[alkahest(Formula)]
pub enum GameMessageFormula {
    Client(ClientMessageFormula),
    Server(ServerMessageFormula),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[alkahest(Serialize<GameMessageFormula>, Deserialize<'_, GameMessageFormula>)]
pub enum GameMessage {
    Client(ClientMessage),
    Server(ServerMessage),
}

#[derive(Debug)]
#[alkahest(Deserialize<'de, GameMessageFormula>)]
pub enum GameMessageRead<'de> {
    Client(ClientMessageRead<'de>),
    Server(ServerMessageRead<'de>),
}

#[alkahest(Formula)]
pub enum ClientMessageFormula {
    ClientData { nickname: Ref<str>, clan: Ref<str> },
    Chat(Ref<str>),
}

#[derive(Debug, PartialEq, Eq, Clone)]
#[alkahest(Serialize<ClientMessageFormula>, Deserialize<'_, ClientMessageFormula>)]
pub enum ClientMessage {
    ClientData { nickname: String, clan: String },
    Chat(String),
}

#[derive(Debug)]
#[alkahest(Deserialize<'de, ClientMessageFormula>)]
pub enum ClientMessageRead<'de> {
    ClientData { nickname: &'de str, clan: &'de str },
    Chat(&'de str),
}

#[alkahest(Formula)]
pub enum ServerMessageFormula {
    ServerData(u64),
    ClientChat { client_id: u64, message: Ref<str> },
}

#[derive(Debug, PartialEq, Eq, Clone)]
#[alkahest(Serialize<ServerMessageFormula>, Deserialize<'_, ServerMessageFormula>)]
pub enum ServerMessage {
    ServerData(u64),
    ClientChat { client_id: u64, message: String },
}

#[derive(Debug)]
#[alkahest(Deserialize<'de, ServerMessageFormula>)]
pub enum ServerMessageRead<'de> {
    ServerData(u64),
    ClientChat { client_id: u64, message: &'de str },
}

#[alkahest(Formula)]
pub struct NetPacketFormula<F> {
    pub game_messages: Vec<F>,
}

#[derive(Debug)]
#[alkahest(for<F: Formula> Serialize<NetPacketFormula<F>> where G: Serialize<F>)]
#[alkahest(for<'de, F: Formula> Deserialize<'de, NetPacketFormula<F>> where G: Deserialize<'de, F>)]
pub struct NetPacket<G> {
    pub game_messages: Vec<G>,
}

#[derive(Debug)]
#[alkahest(for<F: Formula> Serialize<NetPacketFormula<F>> where G: Serialize<[F]>)]
#[alkahest(for<F: Formula> SerializeRef<NetPacketFormula<F>> where G: SerializeRef<[F]>)]
pub struct NetPacketWrite<G> {
    pub game_messages: G,
}

#[derive(Debug)]
#[alkahest(Deserialize<'de, NetPacketFormula<F>> where F: Formula)]
pub struct NetPacketRead<'de, F> {
    pub game_messages: Lazy<'de, [F]>,
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
    let size = write_packet_to_vec::<NetPacketFormula<GameMessageFormula>, _>(
        NetPacketWrite {
            game_messages: SerIter(messages(rng.clone(), LEN)),
        },
        &mut buffer,
    );

    let mut buffer2 = Vec::new();
    let size2 = write_packet_to_vec::<NetPacketFormula<GameMessageFormula>, _>(
        NetPacket {
            game_messages: messages(rng, LEN).collect::<Vec<_>>(),
        },
        &mut buffer2,
    );

    assert_eq!(size, size2);
    assert_eq!(buffer[..size], buffer2[..size]);

    let (packet, _) =
        read_packet::<NetPacketFormula<GameMessageFormula>, NetPacketRead<GameMessageFormula>>(&buffer[..]).unwrap();

    for message in packet.game_messages.iter::<GameMessageRead>() {
        match message.unwrap() {
            GameMessageRead::Client(ClientMessageRead::ClientData { nickname, clan }) => {
                let _ = nickname;
                let _ = clan;
            }
            GameMessageRead::Client(ClientMessageRead::Chat(message)) => {
                let _ = message;
            }
            GameMessageRead::Server(ServerMessageRead::ServerData(data)) => {
                let _ = data;
            }
            GameMessageRead::Server(ServerMessageRead::ClientChat { client_id, message }) => {
                let _ = client_id;
                let _ = message;
            }
        }
    }
}
