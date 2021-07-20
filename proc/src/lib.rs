extern crate proc_macro;

mod schema;

use proc_macro::TokenStream;

/// Proc-macro to derive `Schema` trait for user-defined type.
///
/// This macro requires that type is either `struct` or `enum`.
/// All fields must implement `Schema`. If fields are of generic type, proper bounds must be added.
/// Type must not have any lifetimes.
///
/// Macro generates a number auxiliary types along with trait implementation.\
/// Private type for [`Packed`] associated type named `<InputTypeName>Packed`.\
/// Type for [`Unpacked`] associated type with same visibility as input type named `<InputTypeName>Unpacked`.\
/// Type for with [`Pack`] implementation with same visibility as input type named `<InputTypeName>Pack`.\
/// For enums [`Pack`] implementation is generated for each variant instead named `<InputTypeName><VariantName>Pack`.
///
/// [`Packed`]: ../alkahest/trait.Schema.html#associatedtype.Packed
/// [`Unpacked`]: ../alkahest/trait.SchemaUnpack.html#associatedtype.Unpacked
/// [`Pack`]: ../alkahest/trait.Pack.html
#[proc_macro_derive(Schema)]
pub fn derive_schema(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    match schema::derive_schema(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.into_compile_error().into(),
    }
}
