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

fn is_generic_path<'a>(
    path: &syn::Path,
    params: &(impl Clone + Iterator<Item = &'a syn::TypeParam>),
) -> bool {
    path.segments.iter().any(|seg| {
        if params.clone().any(|p| {
            // if p.ident == "T" {
            //     panic!();
            // }
            p.ident == seg.ident
        }) {
            return true;
        }
        match &seg.arguments {
            syn::PathArguments::AngleBracketed(args) => args.args.iter().any(|arg| match arg {
                syn::GenericArgument::Type(ty) => is_generic_ty(ty, params),
                _ => false,
            }),
            syn::PathArguments::Parenthesized(args) => {
                if let syn::ReturnType::Type(_, ty) = &args.output {
                    if is_generic_ty(ty, params) {
                        return true;
                    }
                }
                args.inputs.iter().any(|ty| is_generic_ty(ty, params))
            }
            syn::PathArguments::None => false,
        }
    })
}

// fn has_type_param<'a>(mut params: impl Iterator<Item = &'a syn::GenericParam>) -> bool {
//     params.any(|param| matches!(param, syn::GenericParam::Type(_)))
// }

fn filter_type_param<'a>(
    params: impl Clone + Iterator<Item = &'a syn::GenericParam>,
) -> impl Clone + Iterator<Item = &'a syn::TypeParam> {
    params.filter_map(|param| match param {
        syn::GenericParam::Type(param) => Some(param),
        _ => None,
    })
}

fn is_generic_ty<'a>(
    ty: &syn::Type,
    params: &(impl Clone + Iterator<Item = &'a syn::TypeParam>),
) -> bool {
    match ty {
        syn::Type::Array(syn::TypeArray { elem, .. })
        | syn::Type::Group(syn::TypeGroup { elem, .. })
        | syn::Type::Paren(syn::TypeParen { elem, .. })
        | syn::Type::Ptr(syn::TypePtr { elem, .. })
        | syn::Type::Reference(syn::TypeReference { elem, .. })
        | syn::Type::Slice(syn::TypeSlice { elem, .. }) => is_generic_ty(elem, params),
        syn::Type::BareFn(syn::TypeBareFn { inputs, output, .. }) => {
            if let syn::ReturnType::Type(_, ty) = output {
                if is_generic_ty(ty, params) {
                    return true;
                }
            }
            inputs.iter().any(|arg| is_generic_ty(&arg.ty, params))
        }
        syn::Type::Path(syn::TypePath { qself, path }) => {
            if let Some(syn::QSelf { ty, .. }) = qself {
                if is_generic_ty(ty, params) {
                    return true;
                }
            }
            is_generic_path(path, params)
        }
        syn::Type::TraitObject(syn::TypeTraitObject { bounds, .. }) => {
            bounds.iter().any(|bound| match bound {
                syn::TypeParamBound::Trait(trait_bound) => {
                    is_generic_path(&trait_bound.path, params)
                }
                _ => false,
            })
        }
        syn::Type::Tuple(syn::TypeTuple { elems, .. }) => {
            elems.iter().any(|ty| is_generic_ty(ty, params))
        }
        _ => false,
    }
}

fn struct_field_order_checks(
    data: &syn::DataStruct,
    variant: Option<&syn::Ident>,
    this: &syn::Ident,
    formula: &syn::Path,
) -> proc_macro2::TokenStream {
    let no_named_fields = syn::punctuated::Punctuated::<syn::Field, syn::Token![,]>::new();

    match &data.fields {
        syn::Fields::Named(fields) => fields.named.iter(),
        _ => no_named_fields.iter(),
    }.enumerate()
    .map(|(idx, field)| {
        let order = match variant {
            None => quote::format_ident!(
                "__ALKAHEST_FORMULA_FIELD_{}_IDX",
                field.ident.as_ref().unwrap(),
            ),
            Some(v) => quote::format_ident!(
                "__ALKAHEST_FORMULA_VARIANT_{}_FIELD_{}_IDX",
                v,
                field.ident.as_ref().unwrap(),
            ),
        };
        let f = field.ident.as_ref().unwrap();
        let error = format!("Field `{this}.{f}` is out of order with formula's");
        quote::quote_spanned!(f.span() => ::alkahest::private::debug_assert_eq!(#idx, #formula::#order, #error);)
    })
    .collect()
}

fn enum_field_order_checks(
    data: &syn::DataEnum,
    this: &syn::Ident,
    formula: &syn::Path,
) -> proc_macro2::TokenStream {
    let no_named_fields = syn::punctuated::Punctuated::<syn::Field, syn::Token![,]>::new();

    data.variants.iter().flat_map(|v| {
        match &v.fields {
            syn::Fields::Named(fields) => fields.named.iter(),
            _ => no_named_fields.iter(),
        }
        .enumerate()
        .map(move |(idx, field)| {
            let f = field.ident.as_ref().unwrap();
            let order = quote::format_ident!(
                "__ALKAHEST_FORMULA_VARIANT_{}_FIELD_{}_IDX",
                v.ident,
                field.ident.as_ref().unwrap(),
            );
            let error = format!("Field `{this}.{f}` is out of order with formula's");
            quote::quote_spanned!(f.span() => ::alkahest::private::debug_assert_eq!(#idx, #formula::#order, #error);)
        })
    }).collect()
}
