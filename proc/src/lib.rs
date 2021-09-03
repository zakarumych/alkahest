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
#[proc_macro_derive(Schema, attributes(alkahest))]
pub fn derive_schema(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    match schema::derive_schema(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.into_compile_error().into(),
    }
}

enum AlkahestAttr {
    Empty,
    Bounds(syn::WhereClause),
    Schema(Option<syn::WhereClause>),
    Owned(Option<syn::WhereClause>),
}

impl syn::parse::Parse for AlkahestAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            Ok(AlkahestAttr::Empty)
        } else {
            if input.peek(syn::Token![where]) {
                let clause = input.parse::<syn::WhereClause>()?;
                Ok(AlkahestAttr::Bounds(clause))
            } else {
                match input.parse::<syn::Ident>()? {
                    ident if ident == "schema" => {
                        if input.peek(syn::token::Paren) {
                            let bounds;
                            syn::parenthesized!(bounds in input);
                            let clause = bounds.parse::<syn::WhereClause>()?;
                            Ok(AlkahestAttr::Schema(Some(clause)))
                        } else {
                            Ok(AlkahestAttr::Schema(None))
                        }
                    }
                    ident if ident == "owned" => {
                        if input.peek(syn::token::Paren) {
                            let bounds;
                            syn::parenthesized!(bounds in input);
                            let clause = bounds.parse::<syn::WhereClause>()?;
                            Ok(AlkahestAttr::Owned(Some(clause)))
                        } else {
                            Ok(AlkahestAttr::Owned(None))
                        }
                    }
                    ident => Err(syn::Error::new_spanned(ident, "Unknown sub-attribute")),
                }
            }
        }
    }
}

fn parse_attrs<'a>(
    attrs: impl Iterator<Item = &'a syn::Attribute>,
) -> syn::Result<Vec<AlkahestAttr>> {
    let mut result = Vec::new();

    for attr in attrs {
        if attr.path.is_ident("alkahest") {
            result.extend(attr.parse_args_with(|stream: syn::parse::ParseStream| {
                stream.parse_terminated::<_, syn::token::Comma>(syn::parse::Parse::parse)
            })?);
        }
    }

    Ok(result)
}

fn get_schema_bounds(attrs: &[AlkahestAttr], generics: &syn::Generics) -> syn::WhereClause {
    for attr in attrs {
        match attr {
            AlkahestAttr::Bounds(bounds) => return bounds.clone(),
            AlkahestAttr::Schema(bounds) => {
                if let Some(bounds) = bounds {
                    return bounds.clone();
                }
            }
            _ => {}
        }
    }
    syn::WhereClause {
        where_token: Default::default(),
        predicates: generics
            .params
            .iter()
            .filter_map(|param| -> Option<syn::WherePredicate> {
                if let syn::GenericParam::Type(param) = param {
                    let ident = &param.ident;
                    Some(syn::parse_quote!(#ident : ::alkahest::Schema))
                } else {
                    None
                }
            })
            .collect(),
    }
}
