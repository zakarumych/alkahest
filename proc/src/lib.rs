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
    params: impl Clone + Iterator<Item = &'a syn::TypeParam>,
) -> bool {
    path.segments.iter().any(|seg| {
        if params.clone().find(|p| p.ident == seg.ident).is_some() {
            return true;
        }
        match &seg.arguments {
            syn::PathArguments::AngleBracketed(args) => args.args.iter().any(|arg| match arg {
                syn::GenericArgument::Type(ty) => is_generic_ty(ty, params.clone()),
                _ => false,
            }),
            syn::PathArguments::Parenthesized(args) => {
                if let syn::ReturnType::Type(_, ty) = &args.output {
                    return is_generic_ty(ty, params.clone());
                }
                args.inputs
                    .iter()
                    .any(|ty| is_generic_ty(ty, params.clone()))
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
    params: impl Clone + Iterator<Item = &'a syn::TypeParam>,
) -> bool {
    match ty {
        syn::Type::Array(syn::TypeArray { elem, .. }) => is_generic_ty(elem, params),
        syn::Type::BareFn(syn::TypeBareFn { inputs, output, .. }) => {
            if let syn::ReturnType::Type(_, ty) = output {
                return is_generic_ty(ty, params);
            }
            inputs
                .iter()
                .any(|arg| is_generic_ty(&arg.ty, params.clone()))
        }
        syn::Type::Group(group) => is_generic_ty(&group.elem, params),
        syn::Type::Paren(paren) => is_generic_ty(&paren.elem, params),
        syn::Type::Path(syn::TypePath { qself, path }) => {
            if let Some(syn::QSelf { ty, .. }) = qself {
                if is_generic_ty(&ty, params.clone()) {
                    return true;
                }
            }
            is_generic_path(&path, params)
        }
        syn::Type::Ptr(syn::TypePtr { elem, .. }) => is_generic_ty(elem, params),
        syn::Type::Reference(syn::TypeReference { elem, .. }) => is_generic_ty(elem, params),
        syn::Type::Slice(syn::TypeSlice { elem, .. }) => is_generic_ty(elem, params),
        syn::Type::TraitObject(syn::TypeTraitObject { bounds, .. }) => {
            bounds.iter().any(|bound| match bound {
                syn::TypeParamBound::Trait(trait_bound) => {
                    is_generic_path(&trait_bound.path, params.clone())
                }
                _ => false,
            })
        }
        syn::Type::Tuple(syn::TypeTuple { elems, .. }) => {
            elems.iter().any(|ty| is_generic_ty(ty, params.clone()))
        }
        _ => false,
    }
}
