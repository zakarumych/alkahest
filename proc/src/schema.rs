use proc_macro2::TokenStream;

pub fn derive(input: proc_macro::TokenStream) -> syn::Result<TokenStream> {
    let input = syn::parse::<syn::DeriveInput>(input)?;
    let ident = &input.ident;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

    match &input.data {
        syn::Data::Union(data) => Err(syn::Error::new_spanned(
            data.union_token,
            "Schema cannot be derived for unions",
        )),
        _ => Ok(
            quote::quote!(impl #impl_generics ::alkahest::Schema for #ident #type_generics #where_clause {}),
        ),
    }
}
