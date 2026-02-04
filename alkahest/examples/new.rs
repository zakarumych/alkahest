use alkahest::{Deserialize, Element, Formula, Mixture, Never, Serialize, alkahest};

// #[alkahest(Formula)]
// struct Parent;

// // #[alkahest]
// mod new {
//     alkahest::include_formulas!("new.alk");
// }

// #[derive(Mixture)]
// struct TryString<S> {
//     a: S,
// }

// const fn is_mixture<T: Mixture>() {}

// const _: () = {
//     is_mixture::<TryString<u8>>();
//     is_mixture::<TryString<String>>();
// };

// #[derive(Serialize)]
// #[alkahest(new::Foo)]
// struct Foo {
//     a: u32,
//     b: u32,
// }

// #[derive(Serialize)]
// #[alkahest(new::Side@Left)]
// struct Left {
//     a: u8,
// }

#[alkahest(Mixture)]
enum Y<A> {
    A(u8),
    B(A),
}

#[alkahest(for<A: Element> Serialize<Y<A>>)]
enum YS {
    A(u8),
    B(Never),
}

#[alkahest(for<'de, A: Formula> Deserialize<'de, Y<A>> where B: Deserialize<'de, A>)]
enum YD<B> {
    A(u8),
    B(B),
}

fn main() {
    // use new::Foo;
}
