use alkahest::{deserialize, serialize, serialized_size, Deserialize, Ref, Schema, Serialize};

#[derive(Schema, Serialize)]
struct X;

#[derive(Schema)]
pub struct Test<T> {
    pub a: u32,
    pub b: T,
    c: X,
}

#[derive(Serialize)]
#[alkahest(schema(Test<u32>))]
pub struct Test2 {
    pub a: u32,
    pub b: u32,
    c: X,
}

fn main() {
    type Schema = (u32, u32, Ref<[u32]>);

    let value = (1, 5, 2..=4);
    let size = serialized_size::<Schema, _>(value);
    println!("size: {}", size);

    let mut buffer = vec![0; size];
    let actual_size = serialize::<Schema, _>((1, 5, 2..=4), &mut buffer).unwrap();
    debug_assert_eq!(actual_size, size);
    let (test_value, _) = deserialize::<Schema, (u32, u32, Vec<u32>)>(&buffer).unwrap();

    println!("{:?}", test_value);
}
