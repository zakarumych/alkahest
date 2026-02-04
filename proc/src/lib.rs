use proc_macro::TokenStream;

extern crate proc_macro;

mod attrs;
mod deserialize;
mod formula;
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

#[proc_macro_derive(Deserialize, attributes(alkahest))]
pub fn deserialize(item: TokenStream) -> TokenStream {
    deserialize::derive(item)
}
