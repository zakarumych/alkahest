extern crate proc_macro;

mod attrs;
mod deserialize;
mod formula;
mod serialize;

use proc_macro::TokenStream;

/// Proc-macro to derive `Formula` trait for user-defined type.
///
/// This macro requires that type is either `struct` or `enum`.
/// All fields must implement `Formula`.
#[proc_macro_derive(Formula, attributes(alkahest))]
pub fn derive_formula(input: TokenStream) -> TokenStream {
    match formula::derive(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

/// Proc-macro to derive `Serialize` trait for user-defined type.
///
/// This macro requires that type is either `struct` or `enum`.
/// All fields must implement `Serialize`.
#[proc_macro_derive(Serialize, attributes(alkahest))]
pub fn derive_serialize(input: TokenStream) -> TokenStream {
    match serialize::derive(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

/// Proc-macro to derive `Deserialize` trait for user-defined type.
///
/// This macro requires that type is either `struct` or `enum`.
/// All fields must implement `Deserialize`.
#[proc_macro_derive(Deserialize, attributes(alkahest))]
pub fn derive_deserialize(input: TokenStream) -> TokenStream {
    match deserialize::derive(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
