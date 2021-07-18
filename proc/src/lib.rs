extern crate proc_macro;

mod schema;

use proc_macro::TokenStream;

/// Proc-macro to automatically implement `Schema` trait for user-defined type.
///
/// Usage:
/// * Ensure that type is a `struct`.
/// * Ensure that all fields implement `Schema`.
/// * Add `#[derive(Schema)]` attribute to the type definition.
#[proc_macro_derive(Schema)]
pub fn derive_schema(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    match schema::derive_schema(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.into_compile_error().into(),
    }
}
