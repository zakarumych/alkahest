use proc_macro2::{Span, TokenStream};

fn make_formula_generics(generics: &syn::Generics) -> syn::Generics {
    let mut formula_generics = generics.clone();
    if generics.type_params().count() > 0 {
        formula_generics
            .make_where_clause()
            .predicates
            .extend(generics.type_params().map(|param| {
                syn::WherePredicate::Type(syn::PredicateType {
                    lifetimes: None,
                    bounded_ty: syn::Type::Path(syn::TypePath {
                        qself: None,
                        path: syn::Path {
                            leading_colon: None,
                            segments: std::iter::once(syn::PathSegment {
                                ident: param.ident.clone(),
                                arguments: syn::PathArguments::None,
                            })
                            .collect(),
                        },
                    }),
                    colon_token: syn::Token![:](Span::call_site()),
                    bounds: std::iter::once(syn::TypeParamBound::Trait(syn::TraitBound {
                        paren_token: None,
                        modifier: syn::TraitBoundModifier::None,
                        lifetimes: None,
                        path: syn::Path {
                            leading_colon: None,
                            segments: std::iter::once(syn::PathSegment {
                                ident: syn::Ident::new("__Alkahest_Element", Span::call_site()),
                                arguments: syn::PathArguments::None,
                            })
                            .collect(),
                        },
                    }))
                    .collect(),
                })
            }));
    }

    formula_generics
}

fn make_size_generics(formula_generics: &syn::Generics) -> syn::Generics {
    let mut size_generics = formula_generics.clone();

    size_generics.lt_token.get_or_insert_default();
    size_generics.gt_token.get_or_insert_default();

    size_generics
        .params
        .push(syn::GenericParam::Const(syn::ConstParam {
            attrs: Vec::new(),
            const_token: syn::Token![const](Span::call_site()),
            ident: syn::Ident::new("__SIZE_BYTES", Span::call_site()),
            colon_token: syn::Token![:](Span::call_site()),
            ty: syn::Type::Path(syn::TypePath {
                qself: None,
                path: syn::Path {
                    leading_colon: None,
                    segments: std::iter::once(syn::PathSegment {
                        ident: syn::Ident::new("u8", Span::call_site()),
                        arguments: syn::PathArguments::None,
                    })
                    .collect(),
                },
            }),
            eq_token: None,
            default: None,
        }));

    size_generics
}

pub fn derive_unit(name: syn::Ident, generics: syn::Generics, tokens: &mut TokenStream) {}
