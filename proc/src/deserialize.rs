use proc_macro2::TokenStream;

use crate::{
    attrs::{parse_attributes, Args, Formula},
    enum_field_order_checks, struct_field_order_checks,
};

fn default_de_lifetime() -> syn::Lifetime {
    syn::Lifetime::new("'__de", proc_macro2::Span::call_site())
}

fn de_lifetime(formula: &mut Formula, generics: &syn::Generics) -> syn::Lifetime {
    match formula.generics.lifetimes().next() {
        None => {
            let lifetime = default_de_lifetime();
            let bounds: syn::punctuated::Punctuated<_, syn::Token![+]> =
                generics.lifetimes().map(|lt| lt.lifetime.clone()).collect();
            let de = syn::LifetimeDef {
                attrs: Vec::new(),
                lifetime: lifetime.clone(),
                colon_token: (!bounds.is_empty()).then(Default::default),
                bounds,
            };
            formula.generics.params.push(de.into());
            lifetime
        }
        Some(first) => first.lifetime.clone(),
    }
}

struct Config {
    formula: Formula,

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
    fn for_struct(
        args: Args,
        data: &syn::DataStruct,
        generics: &syn::Generics,
    ) -> syn::Result<Self> {
        // let non_exhaustive = args.non_exhaustive.is_some();
        match args.deserialize.or(args.common) {
            None => {
                let mut formula = Formula {
                    path: syn::parse_quote!(Self),
                    generics: syn::Generics {
                        lt_token: Some(Default::default()),
                        params: Default::default(),
                        gt_token: Some(Default::default()),
                        where_clause: None,
                    },
                };

                let de = de_lifetime(&mut formula, generics);

                // Add predicates that fields implement
                // `Formula + Deserialize<'__de, #field_type>`
                // Except that last one if `non_exhaustive` is not set.
                let predicates = data.fields.iter().map(|field| -> syn::WherePredicate {
                        let ty = &field.ty;
                        syn::parse_quote! { #ty: ::alkahest::private::Formula + ::alkahest::private::Deserialize<#de, #ty> }
                    });

                formula
                    .generics
                    .make_where_clause()
                    .predicates
                    .extend(predicates);

                Ok(Config {
                    formula,
                    check_fields: false,
                    // non_exhaustive,
                    de,
                })
            }
            Some(mut formula) => {
                let de = de_lifetime(&mut formula, generics);

                Ok(Config {
                    formula,
                    check_fields: true,
                    // non_exhaustive,
                    de,
                })
            }
        }
    }

    fn for_enum(args: Args, data: &syn::DataEnum, generics: &syn::Generics) -> syn::Result<Self> {
        // let non_exhaustive = args.non_exhaustive.is_some();
        match args.deserialize.or(args.common) {
            None => {
                let mut formula = Formula {
                    path: syn::parse_quote!(Self),
                    generics: syn::Generics {
                        lt_token: Some(Default::default()),
                        params: Default::default(),
                        gt_token: Some(Default::default()),
                        where_clause: None,
                    },
                };

                let de = de_lifetime(&mut formula, generics);

                // Add predicates that fields implement
                // `Formula + Deserialize<'__de, #field_type>`
                // Except that last one if `non_exhaustive` is not set.
                let predicates = data.variants.iter().flat_map(|v| v.fields.iter().map(|field| -> syn::WherePredicate {
                        let ty = &field.ty;
                        syn::parse_quote! { #ty: ::alkahest::private::Formula + ::alkahest::private::Deserialize<#de, #ty> }
                    }));

                formula
                    .generics
                    .make_where_clause()
                    .predicates
                    .extend(predicates);

                Ok(Config {
                    formula,
                    check_fields: false,
                    // non_exhaustive,
                    de,
                })
            }
            Some(mut formula) => {
                let de = de_lifetime(&mut formula, generics);

                Ok(Config {
                    formula,
                    check_fields: true,
                    // non_exhaustive,
                    de,
                })
            }
        }
    }
}

pub fn derive(input: proc_macro::TokenStream) -> syn::Result<TokenStream> {
    let input = syn::parse::<syn::DeriveInput>(input)?;
    let args = parse_attributes(&input.attrs)?;

    let ident = &input.ident;

    match input.data {
        syn::Data::Union(_) => Err(syn::Error::new_spanned(
            input,
            "Deserialize cannot be derived for unions",
        )),
        syn::Data::Struct(data) => {
            let cfg = Config::for_struct(args, &data, &input.generics)?;

            let field_checks = if cfg.check_fields {
                struct_field_order_checks(&data, None, &input.ident, &cfg.formula.path)
            } else {
                TokenStream::new()
            };

            let formula_path = &cfg.formula.path;

            let de = cfg.de;

            let mut deserialize_generics = input.generics.clone();

            deserialize_generics.lt_token = deserialize_generics
                .lt_token
                .or(cfg.formula.generics.lt_token);
            deserialize_generics.gt_token = deserialize_generics
                .gt_token
                .or(cfg.formula.generics.gt_token);
            deserialize_generics
                .params
                .extend(cfg.formula.generics.params.into_iter());

            if let Some(where_clause) = cfg.formula.generics.where_clause {
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
                        de.finish()?;

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
                        de.finish()
                    }
                }
            })
        }
        syn::Data::Enum(data) => {
            let cfg = Config::for_enum(args, &data, &input.generics)?;

            let field_checks = if cfg.check_fields {
                enum_field_order_checks(&data, &input.ident, &cfg.formula.path)
            } else {
                TokenStream::new()
            };

            let formula_path = &cfg.formula.path;

            let de = cfg.de;

            let mut deserialize_generics = input.generics.clone();

            deserialize_generics.lt_token = deserialize_generics
                .lt_token
                .or(cfg.formula.generics.lt_token);
            deserialize_generics.gt_token = deserialize_generics
                .gt_token
                .or(cfg.formula.generics.gt_token);
            deserialize_generics
                .params
                .extend(cfg.formula.generics.params.into_iter());

            if let Some(where_clause) = cfg.formula.generics.where_clause {
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
                                    de.finish()?;
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
                                    de.finish()?;
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
                                    de.finish()?;
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
