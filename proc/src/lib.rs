extern crate proc_macro;

mod schema;

use proc_macro::TokenStream;

/// Proc-macro to derive `Schema` trait for user-defined type.
///
/// This macro requires that type is either `struct` or `enum`.
/// All fields must implement `Schema`. If fields are of generic type.
/// Type must not have any lifetimes.
#[proc_macro_derive(Schema, attributes(alkahest))]
pub fn derive_schema(input: TokenStream) -> TokenStream {
    match schema::derive_schema(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
