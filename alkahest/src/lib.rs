#[cfg(feature = "proc")]
pub use alkahest_proc::*;

pub use alkahest_core::*;

#[cfg(test)]
extern crate self as alkahest;

#[cfg(all(feature = "proc", feature = "std", test))]
mod tests {
    use core::fmt;

    use super::{
        manual_size::{deserialize, pack_to_vec, serialize_to_vec, unpack},
        *,
    };

    fn serialize_test<E, T>(value: &T, expected: &[u8])
    where
        E: Element,
        T: Serialize<E::Formula>,
    {
        let mut buf = Vec::new();
        let size = serialize_to_vec::<E, T, 1>(value, &mut buf);
        if &buf[..size] != expected {
            panic!(
                "Serialization failed:
expected {:02x?},
got:     {:02x?}",
                expected, buf
            );
        }
    }

    fn round_trip_test<E, T>(value: &T)
    where
        E: Element,
        T: Serialize<E::Formula> + for<'de> Deserialize<'de, E::Formula> + PartialEq + fmt::Debug,
    {
        let mut buf = Vec::new();
        let size = serialize_to_vec::<E, T, 1>(value, &mut buf);
        match deserialize::<E, T, 1>(&buf[..size]) {
            Ok(deserialized) => {
                if &deserialized != value {
                    panic!(
                        "Round trip failed:
expected: {:?},
got:      {:?}",
                        value, deserialized
                    );
                }
            }
            Err(err) => {
                panic!("Deserialization failed: {:?}", err);
            }
        }
    }

    fn round_trip_packet_test<E, T>(value: &T)
    where
        E: Element,
        T: Serialize<E::Formula> + for<'de> Deserialize<'de, E::Formula> + PartialEq + fmt::Debug,
    {
        let mut buf = Vec::new();
        let pack_size = pack_to_vec::<E, T, 1>(value, &mut buf);

        buf.extend(0..255);

        match unpack::<E, T, 1>(&buf) {
            Ok((deserialized, size)) => {
                if pack_size != size {
                    panic!(
                        "Packet size mismatch:
expected: {},
got:      {}",
                        pack_size, size
                    );
                }

                if &deserialized != value {
                    panic!(
                        "Round trip failed:
expected: {:?},
got:      {:?}",
                        value, deserialized
                    );
                }
            }
            Err(err) => {
                panic!("Deserialization failed: {:?}", err);
            }
        }
    }

    #[test]
    fn test_empty() {
        #[derive(Formula)]
        struct EmptyFormula;

        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        #[alkahest(EmptyFormula)]
        struct Empty;

        serialize_test::<EmptyFormula, _>(&Empty, &[]);

        serialize_test::<Indirect<EmptyFormula>, _>(&Empty, &[0x00]);

        round_trip_test::<EmptyFormula, _>(&Empty);
        round_trip_test::<Indirect<EmptyFormula>, _>(&Empty);
    }

    #[test]
    fn test_u32() {
        #[derive(Formula)]
        struct U32Formula(u32);

        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        #[alkahest(U32Formula)]
        struct U32(u32);

        serialize_test::<U32Formula, _>(&U32(0x12345678), &[0x78, 0x56, 0x34, 0x12]);

        serialize_test::<Indirect<U32Formula>, _>(
            &U32(0x12345678),
            &[0x78, 0x56, 0x34, 0x12, 0x04],
        );

        round_trip_test::<U32Formula, _>(&U32(0x12345678));
        round_trip_test::<Indirect<U32Formula>, _>(&U32(0x12345678));
    }

    #[test]
    fn test_complex() {
        #[derive(Mixture, PartialEq, Debug)]
        struct Empty;

        #[derive(Mixture, PartialEq, Debug)]
        enum Either<A, B> {
            A(A),
            B(B),
        }

        #[derive(Mixture, PartialEq, Debug)]
        struct Complex {
            a: u8,
            b: Vec<u16>,
            c: Vec<Empty>,
            d: Either<u8, String>,
            f: Option<Never>,
            e: u32,
        }

        let value = Complex {
            a: 0x12,
            b: vec![0x3456, 0x789a],
            c: vec![Empty, Empty, Empty],
            d: Either::B("Hello World".to_string()),
            f: None,
            e: 0x12345678,
        };

        serialize_test::<Complex, _>(
            &value,
            &[
                0x9a, 0x78, // b[1]
                0x56, 0x34, // b[0]
                0x02, // length of b
                0x03, // length of c
                b'H', b'e', b'l', b'l', b'o', b' ', b'W', b'o', b'r', b'l', b'd', // d string
                0x0b, // d length
                0x78, 0x56, 0x34, 0x12, // e
                // f uses 0 bytes,
                0x12, // d string heap pointer
                0x01, // d discriminant for Either::B
                0x06, // heap pointer for c
                0x05, // heap pointer for b
                0x12, // a
            ],
        );

        round_trip_test::<Complex, _>(&value);
        round_trip_packet_test::<Complex, _>(&value);
    }
}
