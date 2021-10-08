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

mod kw {
    // syn::custom_keyword!(alkahest);
    // syn::custom_keyword!(schema);
    // syn::custom_keyword!(owned);
}

enum AlkahestAttr {
    Bounds(syn::WhereClause),
    // Schema {
    //     schema: kw::schema,
    //     clause: syn::WhereClause,
    // },
    // Owned {
    //     owned: kw::owned,
    //     clause: Option<syn::WhereClause>,
    // },
}

impl syn::parse::Parse for AlkahestAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(syn::Token![where]) {
            let clause = input.parse::<syn::WhereClause>()?;
            Ok(AlkahestAttr::Bounds(clause))
        } else {
            // if input.peek(kw::schema) {
            //     let schema = input.parse::<kw::schema>()?;
            //     let bounds;
            //     syn::parenthesized!(bounds in input);
            //     let clause = bounds.parse::<syn::WhereClause>()?;
            //     Ok(AlkahestAttr::Schema { schema, clause })
            // } else if input.peek(kw::owned) {
            //     let owned = input.parse::<kw::owned>()?;
            //     if input.peek(syn::token::Paren) {
            //         let bounds;
            //         syn::parenthesized!(bounds in input);
            //         let clause = bounds.parse::<syn::WhereClause>()?;
            //         Ok(AlkahestAttr::Owned {
            //             owned,
            //             clause: Some(clause),
            //         })
            //     } else {
            //         Ok(AlkahestAttr::Owned {
            //             owned,
            //             clause: None,
            //         })
            //     }
            // } else {
            Err(input.error("Expected where clause, `schema` with where clause, or `owner` ident with optional where clause"))
            // }
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

struct AlkahestConfig {
    schema_bounds: syn::WhereClause,
    // owned_bounds: syn::WhereClause,
    // derive_owned: bool,
}

impl AlkahestConfig {
    fn from_input(input: &syn::DeriveInput) -> syn::Result<Self> {
        let attrs = parse_attrs(input.attrs.iter())?;
        Self::new(&attrs, &input.generics)
    }

    fn new(attrs: &[AlkahestAttr], generics: &syn::Generics) -> syn::Result<Self> {
        let mut common_bounds = None;
        let schema_bounds = None;
        // let mut owned_bounds = None;
        // let mut derive_owned = false;

        for attr in attrs {
            match attr {
                AlkahestAttr::Bounds(clause) => {
                    if common_bounds.is_some() {
                        return Err(syn::Error::new_spanned(
                            clause,
                            "Duplicate where clause for alkahest derive",
                        ));
                    }
                    // if schema_bounds.is_some() {
                    //     return Err(syn::Error::new_spanned(
                    //         clause,
                    //         "Redundant where clause for alkahest derive when `Schema` specific is already provided",
                    //     ));
                    // }
                    // if owned_bounds.is_some() {
                    //     return Err(syn::Error::new_spanned(
                    //         clause,
                    //         "Redundant where clause for alkahest derive when `SchemaOwned` specific is already provided",
                    //     ));
                    // }
                    common_bounds = Some(clause);
                } // AlkahestAttr::Schema { schema, clause } => {
                  //     if common_bounds.is_some() {
                  //         return Err(syn::Error::new_spanned(
                  //             schema,
                  //             "Redundant where clause for `alkahest::Schema` derive when common one is already provided",
                  //         ));
                  //     }
                  //     if schema_bounds.is_some() {
                  //         return Err(syn::Error::new_spanned(
                  //             schema,
                  //             "Duplicate where clause for `alkahest::Schema` derive",
                  //         ));
                  //     }

                  //     schema_bounds = Some(clause);
                  // }
                  // AlkahestAttr::Owned { owned, clause } => {
                  //     if derive_owned {
                  //         return Err(syn::Error::new_spanned(
                  //             owned,
                  //             "Redundant `owned` attribute",
                  //         ));
                  //     }

                  //     derive_owned = true;

                  //     if let Some(clause) = clause {
                  //         if common_bounds.is_some() {
                  //             return Err(syn::Error::new_spanned(
                  //                 owned,
                  //                 "Redundant where clause for `alkahest::Owned` derive when common one is already provided",
                  //             ));
                  //         }
                  //         if owned_bounds.is_some() {
                  //             return Err(syn::Error::new_spanned(
                  //                 owned,
                  //                 "Duplicate where clause for `alkahest::Owned` derive",
                  //             ));
                  //         }
                  //         owned_bounds = Some(clause);
                  //     }
                  // }
            }
        }

        Ok(AlkahestConfig {
            // derive_owned,
            schema_bounds: schema_bounds.or(common_bounds).cloned().unwrap_or_else(|| {
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
            }),
            // owned_bounds: owned_bounds.or(common_bounds).cloned().unwrap_or_else(|| {
            //     syn::WhereClause {
            //         where_token: Default::default(),
            //         predicates: generics
            //             .params
            //             .iter()
            //             .filter_map(|param| -> Option<syn::WherePredicate> {
            //                 if let syn::GenericParam::Type(param) = param {
            //                     let ident = &param.ident;
            //                     Some(syn::parse_quote!(#ident : ::alkahest::SchemaOwned))
            //                 } else {
            //                     None
            //                 }
            //             })
            //             .collect(),
            //     }
            // }),
        })
    }
}
