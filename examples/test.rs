use alkahest::{serialize, Ref, Schema, Serialize};

#[derive(Schema)]
pub struct TestStruct {
    pub a: u32,
    pub b: Ref<u32>,
}

#[derive(Serialize)]
#[alkahest(schema(TestStruct))]
pub struct TestStructSerialize {
    pub a: u32,
    pub b: u32,
}

fn main() {
    let mut buffer = [0; 1024];
    let size = serialize::<TestStruct, _>(TestStructSerialize { a: 1, b: 2 }, &mut buffer).unwrap();

    println!("{:?}", &buffer[..size])
}
