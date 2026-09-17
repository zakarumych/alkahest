#![allow(unused)]

use alkahest::{Deserialize, Element, Formula, Indirect, Mixture, Never, Serialize, alkahest};

#[alkahest(Formula)]
struct Parent;

#[alkahest(path = "$/examples/new.alk")] // "$" points to the crate root directory.
mod new; // Requires Rust 1.99+

#[derive(Mixture)]
struct TryString<S> {
    a: S,
}

const fn is_mixture<T: Mixture>() {}

const _: () = {
    is_mixture::<TryString<u8>>();
    is_mixture::<TryString<String>>();
};

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

#[derive(Mixture)]
struct TryIndirect<T> {
    a: Indirect<T>,
}

#[derive(alkahest::Mixture)]
pub enum GameType {
    Survival,
    Creative,
    Adventure,
    Spectator,
}

fn main() {}
