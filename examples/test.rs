use alkahest::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Formula, Serialize, Deserialize)]
struct X;

#[derive(Formula)]
struct Test<T: ?Sized> {
    a: u32,
    b: X,
    c: T,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[alkahest(serialize(for<U: ?Sized> Test<U> where U: Formula, T: Serialize<U>))]
#[alkahest(serialize(owned(for<U: ?Sized> Test<U> where U: Formula, T: SerializeOwned<U>)))]
#[alkahest(deserialize(for<'de, U: ?Sized> Test<U> where U: Formula, T: Deserialize<'de, U>))]
struct TestS<T> {
    a: u32,
    b: X,
    c: T,
}

fn main() {
    let value = TestS {
        a: 1,
        b: X,
        c: vec![2..4, 4..6],
    };

    let size = serialized_size::<Test<[Vec<u32>]>, _>(value.clone());
    println!("size: {}", size);

    let mut buffer = vec![0; size];

    let size = serialize::<Test<[Vec<u32>]>, _>(value.clone(), &mut buffer).unwrap();
    assert_eq!(size, buffer.len());

    let (value, size) = deserialize::<Test<[Vec<u32>]>, TestS<Vec<Vec<u32>>>>(&buffer).unwrap();
    assert_eq!(size, buffer.len());

    assert_eq!(value.a, 1);
    assert_eq!(value.b, X);
    assert_eq!(value.c, vec![vec![2, 3], vec![4, 5]]);

    println!("{:?}", value);
}
