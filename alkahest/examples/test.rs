// #![deny(warnings)]
#![allow(clippy::disallowed_names)]

use alkahest::*;

#[derive(Mixture)]
enum Foo {}

#[allow(unused)]
fn foo(_: Foo) {}

fn main() {}
