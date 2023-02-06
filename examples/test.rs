use alkahest::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Formula, Serialize, Deserialize)]
struct X;

#[derive(Clone, Debug, Formula)]
struct Test<T: ?Sized> {
    a: u32,
    b: X,
    c: T,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[alkahest(serialize(for<U: ?Sized> Test<U> where U: Formula, for<'ser> &'ser T: Serialize<U>))]
#[alkahest(serialize(owned(for<U: ?Sized> Test<U> where U: Formula, T: Serialize<U>)))]
#[alkahest(deserialize(for<'de, U: ?Sized> Test<U> where U: Formula, T: Deserialize<'de, U>))]
struct TestS<T> {
    a: u32,
    b: X,
    c: T,
}

#[derive(Formula)]
enum Test2 {}

fn main() {
    type MyFormula = Test<[Vec<u32>]>;

    let value = TestS {
        a: 1,
        b: X,
        c: vec![2..4, 4..6],
    };

    let size = serialized_size::<MyFormula, _>(value.clone());
    println!("size: {}", size);

    let mut buffer = vec![0; size];

    let size = serialize::<MyFormula, _>(value.clone(), &mut buffer).unwrap();
    assert_eq!(size, buffer.len());

    let (value, size) = deserialize::<MyFormula, TestS<Vec<Vec<u32>>>>(&buffer).unwrap();
    assert_eq!(size, buffer.len());

    assert_eq!(value.a, 1);
    assert_eq!(value.b, X);
    assert_eq!(value.c, vec![vec![2, 3], vec![4, 5]]);

    println!("{:?}", value);
}
