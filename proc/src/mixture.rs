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
    for attr in attrs {
        if attr.path().is_ident("alkahest") {
            let args = attr.parse_args::<MixtureDeriveArgs>()?;
            return Ok(args);
        }
    }

    Ok(MixtureDeriveArgs::default())
}

pub fn derive_impl(
    input: syn::DeriveInput,
    args: MixtureArgs,
) -> syn::Result<proc_macro2::TokenStream> {
    let mut tokens = proc_macro2::TokenStream::new();

    tokens.extend(crate::formula::derive_impl(
        input.clone(),
        args.formula_args(&input.generics),
    )?);

    let (serialize_args, deserialize_args) = args.serialize_deserialize_args(&input.generics);

    tokens.extend(crate::serialize::derive_impl(
        input.clone(),
        serialize_args,
    )?);

    tokens.extend(crate::deserialize::derive_impl(input, deserialize_args)?);

    Ok(tokens)
}
