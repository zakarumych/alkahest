use alkahest::alkahest;

#[alkahest]
mod new {}

fn main() {
    use new::Foo;
}
