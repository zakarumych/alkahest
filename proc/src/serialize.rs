use proc_macro2::TokenStream;

use crate::attrs::{parse_attributes, path_make_expr_style, Args, Formula};

struct Config {
    reference: Option<Formula>,
    owned: Formula,

    variant: Option<syn::Ident>,

    /// Signals if fields should be checked to match on formula.
    /// `false` if `formula` is inferred to `Self`.
    check_fields: bool,
}

impl Config {
    fn for_struct(
        args: Args,
        data: &syn::DataStruct,
        ident: &syn::Ident,
        generics: &syn::Generics,
    ) -> Self {
        let (_, type_generics, _) = generics.split_for_impl();

        match (args.serialize.or(args.common), args.owned) {
            (None, Some(None)) if generics.params.is_empty() => Config {
                reference: None,
                owned: Formula {
                    path: syn::parse_quote!(#ident),
                    generics: Default::default(),
                },
                variant: None,
                check_fields: false,
            },
            (None, None) if generics.params.is_empty() => Config {
                reference: Some(Formula {
                    path: syn::parse_quote!(#ident),
                    generics: syn::Generics {
                        lt_token: None,
                        params: Default::default(),
                        gt_token: None,
                        where_clause: None,
                    },
                }),
                owned: Formula {
                    path: syn::parse_quote!(#ident),
                    generics: syn::Generics {
                        lt_token: None,
                        params: Default::default(),
                        gt_token: None,
                        where_clause: None,
                    },
                },
                variant: None,
                check_fields: false,
            },
            (None, None) => {
                // Add predicates that fields implement
                // `T: Formula + Serialize<T>`
                let predicates = data.fields.iter().map(|field| -> syn::WherePredicate {
                        let ty = &field.ty;
                        syn::parse_quote! { #ty: ::alkahest::private::Formula }
                    }).chain(data.fields.iter().map(|field| -> syn::WherePredicate {
                        let ty = &field.ty;
                        syn::parse_quote! { for<'ser> &'ser #ty: ::alkahest::private::Serialize<#ty> }
                    }))
                    .collect();

                let generics = syn::Generics {
                    lt_token: None,
                    params: Default::default(),
                    gt_token: None,
                    where_clause: Some(syn::WhereClause {
                        where_token: Default::default(),
                        predicates,
                    }),
                };

                let reference = Formula {
                    path: path_make_expr_style(syn::parse_quote!(#ident #type_generics)),
                    generics,
                };

                // Add predicates that fields implement
                // `T: Formula` and `T: Serialize<T>`
                let predicates = data.fields.iter().map(|field| -> syn::WherePredicate {
                        let ty = &field.ty;
                        syn::parse_quote! { #ty: ::alkahest::private::Formula + ::alkahest::private::Serialize<#ty> }
                    })
                    .collect();

                let generics = syn::Generics {
                    lt_token: None,
                    params: Default::default(),
                    gt_token: None,
                    where_clause: Some(syn::WhereClause {
                        where_token: Default::default(),
                        predicates,
                    }),
                };

                let owned = Formula {
                    path: syn::parse_quote!(Self),
                    generics,
                };

                Config {
                    reference: Some(reference),
                    owned,
                    variant: args.variant,
                    check_fields: false,
                }
            }
            (None, Some(None)) => {
                // Add predicates that fields implement
                // `T: Formula` and `T: Serialize<T>`
                let predicates = data
                    .fields
                    .iter()
                    .map(|field| -> syn::WherePredicate {
                        let ty = &field.ty;
                        syn::parse_quote! { #ty: ::alkahest::private::Formula }
                    })
                    .chain(data.fields.iter().map(|field| -> syn::WherePredicate {
                        let ty = &field.ty;
                        syn::parse_quote! { #ty: ::alkahest::private::Serialize<#ty> }
                    }))
                    .collect();

                let generics = syn::Generics {
                    lt_token: None,
                    params: Default::default(),
                    gt_token: None,
                    where_clause: Some(syn::WhereClause {
                        where_token: Default::default(),
                        predicates,
                    }),
                };

                Config {
                    reference: None,
                    owned: Formula {
                        path: syn::parse_quote!(Self),
                        generics,
                    },
                    variant: None,
                    check_fields: true,
                }
            }
            (None, Some(Some(owned))) => Config {
                owned,
                reference: None,
                variant: args.variant,
                check_fields: true,
            },
            (Some(reference), None | Some(None)) => Config {
                reference: Some(reference.clone()),
                owned: reference,
                variant: args.variant,
                check_fields: true,
            },
            (Some(reference), Some(Some(owned))) => Config {
                reference: Some(reference),
                owned,
                variant: args.variant,
                check_fields: true,
            },
        }
    }

    fn for_enum(
        args: Args,
        data: &syn::DataEnum,
        ident: &syn::Ident,
        generics: &syn::Generics,
    ) -> Self {
        let (_, type_generics, _) = generics.split_for_impl();

        let all_fields = data.variants.iter().flat_map(|v| v.fields.iter());

        match (args.serialize.or(args.common), args.owned) {
            (None, Some(None)) if generics.params.is_empty() => Config {
                reference: None,
                owned: Formula {
                    path: syn::parse_quote!(Self),
                    generics: Default::default(),
                },
                variant: None,
                check_fields: false,
            },
            (None, None) if generics.params.is_empty() => Config {
                reference: Some(Formula {
                    path: syn::parse_quote!(#ident),
                    generics: syn::Generics {
                        lt_token: None,
                        params: Default::default(),
                        gt_token: None,
                        where_clause: None,
                    },
                }),
                owned: Formula {
                    path: syn::parse_quote!(Self),
                    generics: syn::Generics {
                        lt_token: None,
                        params: Default::default(),
                        gt_token: None,
                        where_clause: None,
                    },
                },
                variant: None,
                check_fields: false,
            },
            (None, None) => {
                // Add predicates that fields implement
                // `T: Formula + Serialize<T>`
                let predicates = all_fields.clone().map(|field| -> syn::WherePredicate {
                        let ty = &field.ty;
                        syn::parse_quote! { #ty: ::alkahest::private::Formula }
                    }).chain(all_fields.clone().map(|field| -> syn::WherePredicate {
                        let ty = &field.ty;
                        syn::parse_quote! { for<'ser> &'ser #ty: ::alkahest::private::Serialize<<#ty as ::alkahest::private::Formula>> }
                    }))
                    .collect();

                let generics = syn::Generics {
                    lt_token: None,
                    params: Default::default(),
                    gt_token: None,
                    where_clause: Some(syn::WhereClause {
                        where_token: Default::default(),
                        predicates,
                    }),
                };

                let reference = Formula {
                    path: path_make_expr_style(syn::parse_quote!(#ident #type_generics)),
                    generics,
                };

                // Add predicates that fields implement
                // `T: Formula` and `T: Serialize<T>`
                let predicates = all_fields.clone().map(|field| -> syn::WherePredicate {
                        let ty = &field.ty;
                        syn::parse_quote! { #ty: ::alkahest::private::Formula + ::alkahest::private::Serialize<<#ty as ::alkahest::private::Formula>> }
                    })
                    .collect();

                let generics = syn::Generics {
                    lt_token: None,
                    params: Default::default(),
                    gt_token: None,
                    where_clause: Some(syn::WhereClause {
                        where_token: Default::default(),
                        predicates,
                    }),
                };

                let owned = Formula {
                    path: syn::parse_quote!(Self),
                    generics,
                };

                Config {
                    reference: Some(reference),
                    owned,
                    variant: args.variant,
                    check_fields: false,
                }
            }
            (None, Some(None)) => {
                // Add predicates that fields implement
                // `T: Formula` and `T: Serialize<T>`
                let predicates = all_fields
                    .clone()
                    .map(|field| -> syn::WherePredicate {
                        let ty = &field.ty;
                        syn::parse_quote! { #ty: ::alkahest::private::Formula }
                    })
                    .chain(all_fields.clone().map(|field| -> syn::WherePredicate {
                        let ty = &field.ty;
                        syn::parse_quote! { #ty: ::alkahest::private::Serialize<#ty> }
                    }))
                    .collect();

                let generics = syn::Generics {
                    lt_token: None,
                    params: Default::default(),
                    gt_token: None,
                    where_clause: Some(syn::WhereClause {
                        where_token: Default::default(),
                        predicates,
                    }),
                };

                Config {
                    reference: None,
                    owned: Formula {
                        path: syn::parse_quote!(Self),
                        generics,
                    },
                    variant: None,
                    check_fields: true,
                }
            }
            (None, Some(Some(owned))) => Config {
                owned,
                reference: None,
                variant: args.variant,
                check_fields: true,
            },
            (Some(reference), None | Some(None)) => Config {
                reference: Some(reference.clone()),
                owned: reference,
                variant: args.variant,
                check_fields: true,
            },
            (Some(reference), Some(Some(owned))) => Config {
                reference: Some(reference),
                owned,
                variant: args.variant,
                check_fields: true,
            },
        }
    }
}

pub fn derive(input: proc_macro::TokenStream) -> syn::Result<TokenStream> {
    let input = syn::parse::<syn::DeriveInput>(input)?;
    let args = parse_attributes(&input.attrs)?;

    let ident = &input.ident;
    let generics = &input.generics;
    let (_impl_generics, type_generics, _where_clause) = generics.split_for_impl();

    match input.data {
        syn::Data::Union(_) => Err(syn::Error::new_spanned(
            input,
            "Serialize cannot be derived for unions",
        )),
        syn::Data::Struct(data) => {
            let cfg = Config::for_struct(args, &data, ident, generics);

            let field_count = data.fields.len();

            let field_names_order_check = match (cfg.check_fields, &data.fields) {
                (true, syn::Fields::Named(_)) => data
                    .fields
                    .iter()
                    .map(|field| match &cfg.variant {
                        None => quote::format_ident!(
                            "__ALKAHEST_FORMULA_FIELD_{}_IDX",
                            field.ident.as_ref().unwrap(),
                        ),
                        Some(v) => quote::format_ident!(
                            "__ALKAHEST_FORMULA_VARIANT_{}_FIELD_{}_IDX",
                            v,
                            field.ident.as_ref().unwrap(),
                        ),
                    })
                    .collect(),
                _ => Vec::new(),
            };

            let field_ids_check = match (cfg.check_fields, &data.fields) {
                (true, syn::Fields::Named(_)) => (0..data.fields.len()).collect(),
                _ => Vec::new(),
            };

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

            let with_variant = match &cfg.variant {
                None => quote::quote! {},
                Some(v) => quote::quote! { :: #v },
            };

            let mut tokens = TokenStream::new();
            {
                let formula_path = &cfg.owned.path;
                let field_count_check = match (cfg.check_fields, &cfg.variant) {
                    (false, None) => quote::quote! {},
                    (false, Some(_)) => unreachable!(),
                    (true, None) => quote::quote! {
                        let _: [(); #field_count] = #formula_path::__ALKAHEST_FORMULA_FIELD_COUNT;
                    },
                    (true, Some(v)) => {
                        let ident =
                            quote::format_ident!("__ALKAHEST_FORMULA_VARIANT_{}_FIELD_COUNT", v);
                        quote::quote! {
                            let _: [(); #field_count] = #formula_path::#ident;
                        }
                    }
                };

                let write_variant = match &cfg.variant {
                    None => quote::quote! {},
                    Some(v) => {
                        let variant_name_idx =
                            quote::format_ident!("__ALKAHEST_FORMULA_VARIANT_{}_IDX", v);
                        quote::quote! { ser.write_value::<u32, u32>(#formula_path::#variant_name_idx)?; }
                    }
                };

                let mut generics = input.generics.clone();

                generics.lt_token = generics.lt_token.or(cfg.owned.generics.lt_token);
                generics.gt_token = generics.gt_token.or(cfg.owned.generics.gt_token);
                generics
                    .params
                    .extend(cfg.owned.generics.params.into_iter());

                if let Some(where_clause) = cfg.owned.generics.where_clause {
                    generics
                        .make_where_clause()
                        .predicates
                        .extend(where_clause.predicates);
                }

                let (impl_generics, _type_generics, where_clause) = generics.split_for_impl();

                tokens.extend(quote::quote! {
                    impl #impl_generics ::alkahest::private::Serialize<#formula_path> for #ident #type_generics #where_clause {
                        #[inline(always)]
                        fn serialize<S>(self, ser: impl ::alkahest::private::Into<S>) -> ::alkahest::private::Result<S::Ok, S::Error>
                        where
                            S: ::alkahest::private::Serializer
                        {
                            // Checks compilation of code in the block.
                            #[allow(unused)]
                            {
                                #(let _: [(); #field_ids_check] = #formula_path::#field_names_order_check;)*
                                #field_count_check
                            }

                            let #ident #bind_names = self;

                            let mut ser = ser.into();
                            #write_variant
                            #(
                                let with_formula = ::alkahest::private::with_formula(|s: &#formula_path| match *s {
                                    #formula_path #with_variant #bind_ref_names => #bound_names,
                                    _ => unreachable!(),
                                });
                                with_formula.write_value(&mut ser, #bound_names)?;
                            )*
                            ser.finish()
                        }
                    }
                });
            }

            if let Some(reference) = cfg.reference {
                let formula_path = &reference.path;
                let mut generics = input.generics.clone();

                let write_variant = match &cfg.variant {
                    None => quote::quote! {},
                    Some(v) => {
                        let variant_name_idx =
                            quote::format_ident!("__ALKAHEST_FORMULA_VARIANT_{}_IDX", v);
                        quote::quote! { ser.write_value::<u32, u32>(#formula_path::#variant_name_idx)?; }
                    }
                };

                generics.lt_token = generics.lt_token.or(reference.generics.lt_token);
                generics.gt_token = generics.gt_token.or(reference.generics.gt_token);
                generics
                    .params
                    .extend(reference.generics.params.into_iter());

                if let Some(where_clause) = reference.generics.where_clause {
                    generics
                        .make_where_clause()
                        .predicates
                        .extend(where_clause.predicates);
                }

                let (impl_generics, _type_generics, where_clause) = generics.split_for_impl();

                tokens.extend(quote::quote! {
                    impl #impl_generics ::alkahest::private::Serialize<#formula_path> for &#ident #type_generics #where_clause {
                        #[inline(always)]
                        fn serialize<S>(self, ser: impl ::alkahest::private::Into<S>) -> ::alkahest::private::Result<S::Ok, S::Error>
                        where
                            S: ::alkahest::private::Serializer
                        {
                            let #ident #bind_ref_names = *self;

                            let mut ser = ser.into();
                            #write_variant
                            #(
                                let with_formula = ::alkahest::private::with_formula(|s: &#formula_path| match *s {
                                    #formula_path #with_variant #bind_ref_names => #bound_names,
                                    _ => unreachable!(),
                                });
                                with_formula.write_value(&mut ser, #bound_names)?;
                            )*
                            ser.finish()
                        }
                    }
                });
            }

            Ok(tokens)
        }
        syn::Data::Enum(data) => {
            let cfg = Config::for_enum(args, &data, ident, generics);

            if let Some(variant) = &cfg.variant {
                return Err(syn::Error::new_spanned(
                    variant,
                    "Variant can be specified only for structs",
                ));
            }

            let field_names_order_checks: Vec<_> = data
                .variants
                .iter()
                .flat_map(|v| match (cfg.check_fields, &v.fields) {
                    (true, syn::Fields::Named(_)) => v
                        .fields
                        .iter()
                        .map(|field| {
                            quote::format_ident!(
                                "__ALKAHEST_FORMULA_VARIANT_{}_FIELD_{}_IDX",
                                v.ident,
                                field.ident.as_ref().unwrap(),
                            )
                        })
                        .collect(),
                    _ => Vec::new(),
                })
                .collect();

            let field_ids_checks: Vec<_> = data
                .variants
                .iter()
                .flat_map(|v| match (cfg.check_fields, &v.fields) {
                    (true, syn::Fields::Named(_)) => (0..v.fields.len()).collect(),
                    _ => Vec::new(),
                })
                .collect();

            let field_count_checks: Vec<syn::Ident> =
                data.variants
                    .iter()
                    .map(|variant| {
                        quote::format_ident!(
                            "__ALKAHEST_FORMULA_VARIANT_{}_FIELD_COUNT",
                            variant.ident,
                        )
                    })
                    .collect();

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

            let variant_name_ids: Vec<syn::Ident> = data
                .variants
                .iter()
                .map(|variant| {
                    quote::format_ident!("__ALKAHEST_FORMULA_VARIANT_{}_IDX", variant.ident,)
                })
                .collect();

            let field_counts: Vec<_> = data.variants.iter().map(|v| v.fields.len()).collect();

            let mut tokens = TokenStream::new();
            {
                let formula_path = &cfg.owned.path;
                let field_count_check = if cfg.check_fields {
                    quote::quote! {
                        #(let _: [(); #field_counts] = #formula_path::#field_count_checks;)*
                    }
                } else {
                    quote::quote! {}
                };

                let mut generics = input.generics.clone();

                generics.lt_token = generics.lt_token.or(cfg.owned.generics.lt_token);
                generics.gt_token = generics.gt_token.or(cfg.owned.generics.gt_token);
                generics
                    .params
                    .extend(cfg.owned.generics.params.into_iter());

                if let Some(where_clause) = cfg.owned.generics.where_clause {
                    generics
                        .make_where_clause()
                        .predicates
                        .extend(where_clause.predicates);
                }

                let (impl_generics, _type_generics, where_clause) = generics.split_for_impl();

                tokens.extend(quote::quote! {
                    impl #impl_generics ::alkahest::private::Serialize<#formula_path> for #ident #type_generics #where_clause {
                        #[inline(always)]
                        fn serialize<S>(self, ser: impl ::alkahest::private::Into<S>) -> ::alkahest::private::Result<S::Ok, S::Error>
                        where
                            S: ::alkahest::private::Serializer
                        {
                            // Checks compilation of code in the block.
                            #[allow(unused)]
                            {
                                #(let _: [(); #field_ids_checks] = #formula_path::#field_names_order_checks;)*
                                #field_count_check
                            }

                            match self {
                                #(
                                    #ident::#variant_names #bind_names => {
                                        let mut ser = ser.into();
                                        ser.write_value::<u32, u32>(#formula_path::#variant_name_ids)?;
                                        #(
                                            let with_formula = ::alkahest::private::with_formula(|s: &#formula_path| match *s {
                                                #[allow(unused_variables)]
                                                #formula_path::#variant_names #bind_ref_names => #bound_names,
                                                _ => unreachable!(),
                                            });
                                            with_formula.write_value(&mut ser, #bound_names)?;
                                        )*
                                        ser.finish()
                                    }
                                )*
                            }
                        }
                    }
                });
            }

            if let Some(reference) = cfg.reference {
                let formula_path = &reference.path;
                let mut generics = input.generics.clone();

                generics.lt_token = generics.lt_token.or(reference.generics.lt_token);
                generics.gt_token = generics.gt_token.or(reference.generics.gt_token);
                generics
                    .params
                    .extend(reference.generics.params.into_iter());

                if let Some(where_clause) = reference.generics.where_clause {
                    generics
                        .make_where_clause()
                        .predicates
                        .extend(where_clause.predicates);
                }

                let (impl_generics, _type_generics, where_clause) = generics.split_for_impl();

                tokens.extend(quote::quote! {
                    impl #impl_generics ::alkahest::private::Serialize<#formula_path> for &#ident #type_generics #where_clause {
                        #[inline(always)]
                        fn serialize<S>(self, ser: impl ::alkahest::private::Into<S>) -> ::alkahest::private::Result<S::Ok, S::Error>
                        where
                            S: ::alkahest::private::Serializer
                        {
                            match *self {
                                #(
                                    #ident::#variant_names #bind_ref_names => {
                                        let mut ser = ser.into();
                                        ser.write_value::<u32, u32>(#formula_path::#variant_name_ids)?;
                                        #(
                                            let with_formula = ::alkahest::private::with_formula(|s: &#formula_path| match *s {
                                                #[allow(unused_variables)]
                                                #formula_path::#variant_names #bind_ref_names => #bound_names,
                                                _ => unreachable!(),
                                            });
                                            with_formula.write_value(&mut ser, #bound_names)?;
                                        )*
                                        ser.finish()
                                    }
                                )*
                            }
                        }
                    }
                });
            }

            Ok(tokens)
        }
    }
}
