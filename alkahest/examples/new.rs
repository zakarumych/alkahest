use alkahest::{Serialize, alkahest};

#[alkahest]
mod new {}

#[derive(Serialize)]
#[alkahest(new::Foo)]
struct Foo {
    a: u32,
    b: u32,
}

#[derive(Serialize)]
#[alkahest(new::Side@Left)]
struct Left {
    a: u8,
}

fn main() {
    use new::Foo;
}
