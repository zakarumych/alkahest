use std::collections::HashSet;

use proc_macro2::TokenStream;

use crate::{
    attrs::DeserializeArgs, enum_field_order_checks, filter_type_param, is_generic_ty,
    struct_field_order_checks,
};

fn default_de_lifetime() -> syn::Lifetime {
    syn::Lifetime::new("'__de", proc_macro2::Span::call_site())
}

fn de_lifetime(
    lifetime: Option<syn::Lifetime>,
    formula_generics: &mut syn::Generics,
    generics: &syn::Generics,
) -> syn::Lifetime {
    match lifetime {
        None => {
            let lifetime = default_de_lifetime();
            let bounds: syn::punctuated::Punctuated<_, syn::Token![+]> =
                generics.lifetimes().map(|lt| lt.lifetime.clone()).collect();
            let de = syn::LifetimeParam {
                attrs: Vec::new(),
                lifetime: lifetime.clone(),
                colon_token: (!bounds.is_empty()).then(Default::default),
                bounds,
            };
            formula_generics
                .params
                .push(syn::GenericParam::Lifetime(de));
            lifetime
        }
        Some(lifetime) => lifetime,
    }
}

struct Config {
    formula: syn::Path,
    generics: syn::Generics,

    /// Signals if fields should be checked to match on formula.
    /// `false` if `formula` is inferred to `Self`.
    check_fields: bool,

    // /// Signals that it can deserialize
    // /// formulas with new fields appended.
    // non_exhaustive: bool,
    /// Deserializer lifetime
    de: syn::Lifetime,
}

impl Config {
    fn for_type(args: DeserializeArgs, data: &syn::Data, generics: &syn::Generics) -> Self {
        match (args.formula, args.generics) {
            (None, None) => {
                let mut formula_generics = syn::Generics {
                    lt_token: Some(<syn::Token![<]>::default()),
                    params: syn::punctuated::Punctuated::default(),
                    gt_token: Some(<syn::Token![>]>::default()),
                    where_clause: None,
                };

                let de = de_lifetime(args.lifetime, &mut formula_generics, generics);

                // Add predicates that fields implement
                // `Formula + Deserialize<'__de, #field_type>`
                // Except that last one if `non_exhaustive` is not set.
                match data {
                    syn::Data::Union(_) => unreachable!(),
                    syn::Data::Struct(data) => {
                        let mut all_generic_field_types: HashSet<_> =
                            data.fields.iter().map(|f| &f.ty).collect();
                        all_generic_field_types.retain(|ty| {
                            is_generic_ty(ty, &filter_type_param(generics.params.iter()))
                        });

                        if !all_generic_field_types.is_empty() {
                            let predicates = all_generic_field_types.iter().map(|&ty| -> syn::WherePredicate {
                                syn::parse_quote! { #ty: ::alkahest::private::Formula + ::alkahest::private::Deserialize<#de, #ty> }
                            });

                            formula_generics
                                .make_where_clause()
                                .predicates
                                .extend(predicates);
                        }
                    }
                    syn::Data::Enum(data) => {
                        let all_fields = data.variants.iter().flat_map(|v| v.fields.iter());

                        let mut all_generic_field_types: HashSet<_> =
                            all_fields.map(|f| &f.ty).collect();
                        all_generic_field_types.retain(|ty| {
                            is_generic_ty(ty, &filter_type_param(generics.params.iter()))
                        });

                        if !all_generic_field_types.is_empty() {
                            let predicates = all_generic_field_types.iter().map(|&ty| -> syn::WherePredicate {
                                syn::parse_quote! { #ty: ::alkahest::private::Formula + ::alkahest::private::Deserialize<#de, #ty> }
                            });

                            formula_generics
                                .make_where_clause()
                                .predicates
                                .extend(predicates);
                        }
                    }
                }

                Config {
                    formula: syn::parse_quote! { Self },
                    generics: formula_generics,
                    check_fields: false,
                    // non_exhaustive,
                    de,
                }
            }
            (None, Some(mut formula_generics)) => {
                let de = de_lifetime(args.lifetime, &mut formula_generics, generics);

                Config {
                    formula: syn::parse_quote! { Self },
                    generics: formula_generics,
                    check_fields: false,
                    // non_exhaustive,
                    de,
                }
            }
            (Some(formula), None) => {
                let mut formula_generics = syn::Generics {
                    lt_token: Some(<syn::Token![<]>::default()),
                    params: syn::punctuated::Punctuated::default(),
                    gt_token: Some(<syn::Token![>]>::default()),
                    where_clause: None,
                };

                let de = de_lifetime(args.lifetime, &mut formula_generics, generics);

                Config {
                    formula,
                    generics: formula_generics,
                    check_fields: false,
                    de,
                }
            }
            (Some(formula), Some(mut formula_generics)) => {
                let de = de_lifetime(args.lifetime, &mut formula_generics, generics);

                Config {
                    formula,
                    generics: formula_generics,
                    check_fields: true,
                    de,
                }
            }
        }
    }
}

#[allow(clippy::too_many_lines)]
pub fn derive(args: DeserializeArgs, input: &syn::DeriveInput) -> syn::Result<TokenStream> {
    let ident = &input.ident;

    let cfg = Config::for_type(args, &input.data, &input.generics);

    match &input.data {
        syn::Data::Union(_) => Err(syn::Error::new_spanned(
            input,
            "Deserialize cannot be derived for unions",
        )),
        syn::Data::Struct(data) => {
            let field_checks = if cfg.check_fields {
                struct_field_order_checks(data, None, &input.ident, &cfg.formula)
            } else {
                TokenStream::new()
            };

            let formula_path = &cfg.formula;

            let de = cfg.de;

            let mut deserialize_generics = input.generics.clone();

            deserialize_generics.lt_token = deserialize_generics.lt_token.or(cfg.generics.lt_token);
            deserialize_generics.gt_token = deserialize_generics.gt_token.or(cfg.generics.gt_token);
            deserialize_generics
                .params
                .extend(cfg.generics.params.into_iter());

            if let Some(where_clause) = cfg.generics.where_clause {
                deserialize_generics
                    .make_where_clause()
                    .predicates
                    .extend(where_clause.predicates);
            }

            let field_ids: Vec<_> = (0..data.fields.len()).collect();

            let bound_names = data
                .fields
                .iter()
                .enumerate()
                .map(|(idx, field)| match &field.ident {
                    Some(ident) => ident.clone(),
                    None => quote::format_ident!("_{}", idx),
                })
                .collect::<Vec<_>>();

            let bind_names = match &data.fields {
                syn::Fields::Named(fields) => {
                    let names = fields
                        .named
                        .iter()
                        .map(|field| field.ident.as_ref().unwrap().clone());

                    quote::quote! {
                        { #(#names),* }
                    }
                }
                syn::Fields::Unnamed(fields) => {
                    let names = fields
                        .unnamed
                        .iter()
                        .enumerate()
                        .map(|(idx, _)| quote::format_ident!("_{}", idx));

                    quote::quote! {
                        ( #(#names),* )
                    }
                }
                syn::Fields::Unit => quote::quote! {},
            };

            let bind_ref_names = match &data.fields {
                syn::Fields::Named(fields) => {
                    let names = fields
                        .named
                        .iter()
                        .map(|field| field.ident.as_ref().unwrap().clone());

                    quote::quote! {
                        { #(ref #names),* }
                    }
                }
                syn::Fields::Unnamed(fields) => {
                    let names = fields
                        .unnamed
                        .iter()
                        .enumerate()
                        .map(|(idx, _)| quote::format_ident!("_{}", idx));

                    quote::quote! {
                        ( #(ref #names),* )
                    }
                }
                syn::Fields::Unit => quote::quote! {},
            };

            let bind_ref_mut_names = match &data.fields {
                syn::Fields::Named(fields) => {
                    let names = fields
                        .named
                        .iter()
                        .map(|field| field.ident.as_ref().unwrap().clone());

                    quote::quote! {
                        { #(ref mut #names),* }
                    }
                }
                syn::Fields::Unnamed(fields) => {
                    let names = fields
                        .unnamed
                        .iter()
                        .enumerate()
                        .map(|(idx, _)| quote::format_ident!("_{}", idx));

                    quote::quote! {
                        ( #(ref mut #names),* )
                    }
                }
                syn::Fields::Unit => quote::quote! {},
            };

            let field_count = data.fields.len();

            let (_impl_generics, type_generics, _where_clause) = input.generics.split_for_impl();
            let (impl_deserialize_generics, _type_deserialize_generics, where_serialize_clause) =
                deserialize_generics.split_for_impl();
            Ok(quote::quote! {
                impl #impl_deserialize_generics ::alkahest::private::Deserialize<#de, #formula_path> for #ident #type_generics #where_serialize_clause {
                    #[inline(always)]
                    fn deserialize(mut de: ::alkahest::private::Deserializer<#de>) -> ::alkahest::private::Result<Self, ::alkahest::private::DeserializeError> {
                        #field_checks

                        #(
                            let with_formula = ::alkahest::private::with_formula(|s: &#formula_path| match *s {
                                #formula_path #bind_ref_names => #bound_names,
                                _ => unreachable!(),
                            });
                            let #bound_names = with_formula.read_field(&mut de, #field_count == 1 + #field_ids)?;
                        )*
                        // #consume_tail
                        // de.finish()?;

                        let value = #ident #bind_names;
                        ::alkahest::private::Result::Ok(value)
                    }

                    #[inline(always)]
                    fn deserialize_in_place(&mut self, mut de: ::alkahest::private::Deserializer<#de>) -> Result<(), ::alkahest::private::DeserializeError> {
                        #field_checks

                        let #ident #bind_ref_mut_names = *self;

                        #(
                            let with_formula = ::alkahest::private::with_formula(|s: &#formula_path| match *s {
                                #formula_path #bind_ref_names => #bound_names,
                                _ => unreachable!(),
                            });
                            with_formula.read_in_place(#bound_names, &mut de, #field_count == 1 + #field_ids)?;
                        )*
                        // #consume_tail
                        // de.finish()?;
                        ::alkahest::private::Result::Ok(())
                    }
                }
            })
        }
        syn::Data::Enum(data) => {
            let field_checks = if cfg.check_fields {
                enum_field_order_checks(data, &input.ident, &cfg.formula)
            } else {
                TokenStream::new()
            };

            let formula_path = &cfg.formula;

            let de = cfg.de;

            let mut deserialize_generics = input.generics.clone();

            deserialize_generics.lt_token = deserialize_generics.lt_token.or(cfg.generics.lt_token);
            deserialize_generics.gt_token = deserialize_generics.gt_token.or(cfg.generics.gt_token);
            deserialize_generics
                .params
                .extend(cfg.generics.params.into_iter());

            if let Some(where_clause) = cfg.generics.where_clause {
                deserialize_generics
                    .make_where_clause()
                    .predicates
                    .extend(where_clause.predicates);
            }

            let field_ids: Vec<Vec<_>> = data
                .variants
                .iter()
                .map(|v| (0..v.fields.len()).collect())
                .collect();

            let field_counts: Vec<_> = data.variants.iter().map(|v| v.fields.len()).collect();

            let variant_names = data.variants.iter().map(|v| &v.ident).collect::<Vec<_>>();

            let bound_names = data
                .variants
                .iter()
                .map(|v| {
                    v.fields
                        .iter()
                        .enumerate()
                        .map(|(idx, field)| match &field.ident {
                            Some(ident) => ident.clone(),
                            None => quote::format_ident!("_{}", idx),
                        })
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();

            let bind_names = data
                .variants
                .iter()
                .map(|v| match v.fields {
                    syn::Fields::Named(_) => {
                        let names = v
                            .fields
                            .iter()
                            .map(|field| field.ident.as_ref().unwrap().clone());

                        quote::quote! {
                            { #(#names),* }
                        }
                    }
                    syn::Fields::Unnamed(_) => {
                        let names = v
                            .fields
                            .iter()
                            .enumerate()
                            .map(|(idx, _)| quote::format_ident!("_{}", idx));

                        quote::quote! {
                            ( #(#names),* )
                        }
                    }
                    syn::Fields::Unit => quote::quote! {},
                })
                .collect::<Vec<_>>();

            let bind_ref_names = data
                .variants
                .iter()
                .map(|v| match v.fields {
                    syn::Fields::Named(_) => {
                        let names = v
                            .fields
                            .iter()
                            .map(|field| field.ident.as_ref().unwrap().clone());

                        quote::quote! {
                            { #(ref #names),* }
                        }
                    }
                    syn::Fields::Unnamed(_) => {
                        let names = v
                            .fields
                            .iter()
                            .enumerate()
                            .map(|(idx, _)| quote::format_ident!("_{}", idx));

                        quote::quote! {
                            ( #(ref #names),* )
                        }
                    }
                    syn::Fields::Unit => quote::quote! {},
                })
                .collect::<Vec<_>>();

            let bind_ref_mut_names = data
                .variants
                .iter()
                .map(|v| match v.fields {
                    syn::Fields::Named(_) => {
                        let names = v
                            .fields
                            .iter()
                            .map(|field| field.ident.as_ref().unwrap().clone());

                        quote::quote! {
                            { #(ref mut #names),* }
                        }
                    }
                    syn::Fields::Unnamed(_) => {
                        let names = v
                            .fields
                            .iter()
                            .enumerate()
                            .map(|(idx, _)| quote::format_ident!("_{}", idx));

                        quote::quote! {
                            ( #(ref mut #names),* )
                        }
                    }
                    syn::Fields::Unit => quote::quote! {},
                })
                .collect::<Vec<_>>();

            let variant_name_ids: Vec<syn::Ident> = data
                .variants
                .iter()
                .map(|variant| {
                    quote::format_ident!("__ALKAHEST_FORMULA_VARIANT_{}_IDX", variant.ident,)
                })
                .collect();

            let (_impl_generics, type_generics, _where_clause) = input.generics.split_for_impl();
            let (impl_deserialize_generics, _type_deserialize_generics, where_serialize_clause) =
                deserialize_generics.split_for_impl();
            Ok(quote::quote! {
                impl #impl_deserialize_generics ::alkahest::private::Deserialize<#de, #formula_path> for #ident #type_generics #where_serialize_clause {
                    #[inline(always)]
                    fn deserialize(mut de: ::alkahest::private::Deserializer<#de>) -> ::alkahest::private::Result<Self, ::alkahest::private::DeserializeError> {
                        #field_checks

                        let variant_idx = de.read_value::<::alkahest::private::u32, _>(false)?;
                        match variant_idx {
                            #(
                                #formula_path::#variant_name_ids => {
                                    #(
                                        let with_formula = ::alkahest::private::with_formula(|s: &#formula_path| match *s {
                                            #[allow(unused_variables)]
                                            #formula_path::#variant_names #bind_ref_names => #bound_names,
                                            _ => unreachable!(),
                                        });
                                        let #bound_names = with_formula.read_field(&mut de, #field_counts == 1 + #field_ids)?;
                                    )*
                                    // #consume_tail
                                    // de.finish()?;
                                    ::alkahest::private::Result::Ok(#ident::#variant_names #bind_names)
                                }
                            )*
                            invalid => ::alkahest::private::Result::Err(::alkahest::private::DeserializeError::WrongVariant(invalid)),
                        }
                    }

                    #[inline(always)]
                    fn deserialize_in_place(&mut self, mut de: ::alkahest::private::Deserializer<#de>) -> Result<(), ::alkahest::private::DeserializeError> {
                        #field_checks

                        let variant_idx = de.read_value::<::alkahest::private::u32, _>(false)?;
                        match (variant_idx, self) {
                            #(
                                (#formula_path::#variant_name_ids, #ident::#variant_names #bind_ref_mut_names) => {
                                    #(
                                        let with_formula = ::alkahest::private::with_formula(|s: &#formula_path| match *s {
                                            #[allow(unused_variables)]
                                            #formula_path::#variant_names #bind_ref_names => #bound_names,
                                            _ => unreachable!(),
                                        });
                                        with_formula.read_in_place(#bound_names, &mut de, #field_counts == 1 + #field_ids)?;
                                    )*
                                    // #consume_tail
                                    // de.finish()?;
                                    ::alkahest::private::Result::Ok(())
                                }
                            )*
                            #(
                                (#formula_path::#variant_name_ids, me) => {
                                    #(
                                        let with_formula = ::alkahest::private::with_formula(|s: &#formula_path| match *s {
                                            #[allow(unused_variables)]
                                            #formula_path::#variant_names #bind_ref_names => #bound_names,
                                            _ => unreachable!(),
                                        });
                                        let #bound_names = with_formula.read_field(&mut de, #field_counts == 1 + #field_ids)?;
                                    )*
                                    // #consume_tail
                                    // de.finish()?;
                                    *me = #ident::#variant_names #bind_names;
                                    ::alkahest::private::Result::Ok(())
                                }
                            )*
                            (invalid, _) => ::alkahest::private::Result::Err(::alkahest::private::DeserializeError::WrongVariant(invalid)),
                        }
                    }
                }
            })
        }
    }
}
