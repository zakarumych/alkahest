use crate::args::{MixtureArgs, MixtureDeriveArgs};

pub(crate) fn derive(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(item as syn::DeriveInput);

    let args = match get_args(&input.attrs) {
        Ok(args) => MixtureArgs::from_derive(args),
        Err(err) => {
            return proc_macro::TokenStream::from(err.to_compile_error());
        }
    };

    match derive_impl(input, args) {
        Ok(tokens) => tokens.into(),
        Err(err) => proc_macro::TokenStream::from(err.to_compile_error()),
    }
}

fn get_args(attrs: &[syn::Attribute]) -> syn::Result<MixtureDeriveArgs> {
    crate::args::derive_args(attrs)
}

pub fn derive_impl(
    input: syn::DeriveInput,
    mut args: MixtureArgs,
) -> syn::Result<proc_macro2::TokenStream> {
    let mut tokens = proc_macro2::TokenStream::new();

    tokens.extend(crate::formula::derive_impl(
        input.clone(),
        args.formula_args(&input.generics),
    )?);

    if args.where_clause.is_none() {
        let fields: Vec<_> = match &input.data {
            syn::Data::Struct(data) => data.fields.iter().collect(),
            syn::Data::Enum(data) => data.variants.iter().flat_map(|v| v.fields.iter()).collect(),
            syn::Data::Union(_) => Vec::new(),
        };
        let mut generics = syn::Generics::default();
        for field in fields {
            if !crate::formula::needs_field_bound(&field.ty, &input.generics, &input.ident) {
                continue;
            }
            let ty = &field.ty;
            let version = crate::args::field_version(&field.attrs)?;
            generics
                .make_where_clause()
                .predicates
                .push(syn::parse_quote!(#ty: ::alkahest::Component<#version>));
        }
        args.where_clause = generics.where_clause;
    }
    let (serialize_args, deserialize_args) = args.serialize_deserialize_args(&input.generics);

    tokens.extend(crate::serialize::derive_impl(
        input.clone(),
        serialize_args,
    )?);

    tokens.extend(crate::deserialize::derive_impl(input, deserialize_args)?);

    Ok(tokens)
}
