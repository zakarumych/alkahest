use proc_macro::TokenStream;

extern crate proc_macro;

mod attrs;
mod module;
mod serialize;

#[proc_macro_attribute]
pub fn alkahest(attr: TokenStream, item: TokenStream) -> TokenStream {
    module::alkahest(attr, item)
}

#[proc_macro_derive(Serialize, attributes(alkahest))]
pub fn serialize(item: TokenStream) -> TokenStream {
    serialize::derive(item)
}
