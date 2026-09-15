#![cfg(feature = "proc")]

use alkahest::{Deserialize, Formula, Mixture, Serialize, small};

#[derive(Mixture, Debug, PartialEq)]
struct Named {
    first: Option<u32>,
    middle: u16,
    last: Option<u32>,
}

#[derive(Mixture, Debug, PartialEq)]
struct Unnamed(Option<u32>, u16, Option<u32>);

#[derive(Mixture, Debug, PartialEq)]
enum Variants {
    Named {
        first: Option<u32>,
        middle: u16,
        last: Option<u32>,
    },
    Unnamed(Option<u32>, u16, Option<u32>),
}

fn check<T>(value: &T, expected_size: usize)
where
    T: Formula + Serialize<T> + for<'de> Deserialize<'de, T> + PartialEq + std::fmt::Debug,
{
    assert_eq!(small::serialized_size::<T, _>(value), expected_size);
    for capacity in [expected_size, expected_size + 16] {
        let mut bytes = vec![0; capacity];
        let written = small::serialize::<T, _>(value, &mut bytes).expect("buffer fits");
        assert_eq!(written, expected_size);
        let decoded = small::deserialize::<T, T>(&bytes[..written]).expect("valid serialization");
        assert_eq!(&decoded, value);
    }

    let packet_size = small::pack_size::<T, _>(value);
    assert_eq!(packet_size, expected_size + 1);
    for capacity in [packet_size, packet_size + 16] {
        let mut bytes = vec![0; capacity];
        assert_eq!(small::pack::<T, _>(value, &mut bytes), Ok(packet_size));
        let (decoded, consumed) = small::unpack::<T, T>(&bytes).expect("valid packet");
        assert_eq!(consumed, packet_size);
        assert_eq!(&decoded, value);
    }
}

#[test]
fn derived_struct_hints_pad_nonfinal_fields_only() {
    for first in [None, Some(11)] {
        for last in [None, Some(22)] {
            let expected_size = 5 + 2 + if last.is_some() { 5 } else { 1 };
            check(
                &Named {
                    first,
                    middle: 33,
                    last,
                },
                expected_size,
            );
            check(&Unnamed(first, 33, last), expected_size);
        }
    }
}

#[test]
fn derived_enum_hints_pad_nonfinal_fields_only() {
    for first in [None, Some(11)] {
        for last in [None, Some(22)] {
            let expected_size = 1 + 5 + 2 + if last.is_some() { 5 } else { 1 };
            check(
                &Variants::Named {
                    first,
                    middle: 33,
                    last,
                },
                expected_size,
            );
            check(&Variants::Unnamed(first, 33, last), expected_size);
        }
    }
}
