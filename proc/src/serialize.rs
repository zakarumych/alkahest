use proc_macro2::TokenStream;

use crate::attrs::{parse_attributes, Args, Formula};

struct Config {
    reference: Option<Formula>,
    owned: Option<Formula>,

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
                owned: Some(Formula {
                    ty: syn::parse_quote!(Self),
                    generics: Default::default(),
                }),
                variant: None,
                check_fields: false,
            },
            (None, None) if generics.params.is_empty() => Config {
                reference: Some(Formula {
                    ty: syn::parse_quote!(#ident #type_generics),
                    generics: syn::Generics {
                        lt_token: None,
                        params: Default::default(),
                        gt_token: None,
                        where_clause: None,
                    },
                }),
                owned: Some(Formula {
                    ty: syn::parse_quote!(Self),
                    generics: syn::Generics {
                        lt_token: None,
                        params: Default::default(),
                        gt_token: None,
                        where_clause: None,
                    },
                }),
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
                    ty: syn::parse_quote!(Self),
                    generics,
                };

                // Add predicates that fields implement
                // `T: Formula` and `T: Serialize<T>`
                let predicates = data.fields.iter().map(|field| -> syn::WherePredicate {
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
                    ty: syn::parse_quote!(Self),
                    generics,
                };

                Config {
                    reference: Some(reference),
                    owned: Some(owned),
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
                    owned: Some(Formula {
                        ty: syn::parse_quote!(Self),
                        generics,
                    }),
                    variant: None,
                    check_fields: true,
                }
            }
            (None, Some(Some(owned))) => Config {
                owned: Some(owned),
                reference: None,
                variant: args.variant,
                check_fields: true,
            },
            (Some(reference), None | Some(None)) => Config {
                reference: Some(reference.clone()),
                owned: None,
                variant: args.variant,
                check_fields: true,
            },
            (Some(reference), Some(Some(owned))) => Config {
                reference: Some(reference),
                owned: Some(owned),
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
                owned: Some(Formula {
                    ty: syn::parse_quote!(Self),
                    generics: Default::default(),
                }),
                variant: None,
                check_fields: false,
            },
            (None, None) if generics.params.is_empty() => Config {
                reference: Some(Formula {
                    ty: syn::parse_quote!(#ident #type_generics),
                    generics: syn::Generics {
                        lt_token: None,
                        params: Default::default(),
                        gt_token: None,
                        where_clause: None,
                    },
                }),
                owned: Some(Formula {
                    ty: syn::parse_quote!(Self),
                    generics: syn::Generics {
                        lt_token: None,
                        params: Default::default(),
                        gt_token: None,
                        where_clause: None,
                    },
                }),
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
                    ty: syn::parse_quote!(Self),
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
                    ty: syn::parse_quote!(Self),
                    generics,
                };

                Config {
                    reference: Some(reference),
                    owned: Some(owned),
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
                    owned: Some(Formula {
                        ty: syn::parse_quote!(Self),
                        generics,
                    }),
                    variant: None,
                    check_fields: true,
                }
            }
            (None, Some(Some(owned))) => Config {
                owned: Some(owned),
                reference: None,
                variant: args.variant,
                check_fields: true,
            },
            (Some(reference), None | Some(None)) => Config {
                reference: Some(reference.clone()),
                owned: None,
                variant: args.variant,
                check_fields: true,
            },
            (Some(reference), Some(Some(owned))) => Config {
                reference: Some(reference),
                owned: Some(owned),
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

            if cfg.variant.is_some() {
                unimplemented!("Add support for enums");
            }

            let field_count = data.fields.len();

            let field_names_check = match (cfg.check_fields, &data.fields) {
                (true, syn::Fields::Named(_)) => data
                    .fields
                    .iter()
                    .map(|field| {
                        quote::format_ident!(
                            "__alkahest_formula_field_{}_idx_is",
                            field.ident.as_ref().unwrap(),
                        )
                    })
                    .collect(),
                _ => Vec::new(),
            };

            let field_ids_check = match (cfg.check_fields, &data.fields) {
                (true, syn::Fields::Named(_)) => (0..data.fields.len()).collect(),
                _ => Vec::new(),
            };

            let field_names = data
                .fields
                .iter()
                .enumerate()
                .map(|(index, field)| match &field.ident {
                    Some(ident) => syn::Member::from(ident.clone()),
                    None => syn::Member::from(index),
                })
                .collect::<Vec<_>>();

            match (cfg.reference, cfg.owned) {
                (None, None) => unreachable!(),
                (Some(reference), None) => {
                    let formula_type = &reference.ty;
                    let field_count_check = if cfg.check_fields {
                        quote::quote! {
                            let _: [(); #field_count] = <#formula_type>::__alkahest_formula_field_count();
                        }
                    } else {
                        quote::quote! {}
                    };

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
                    Ok(quote::quote! {
                        impl #impl_generics ::alkahest::private::Serialize<#formula_type> for #ident #type_generics #where_clause {
                            fn serialize<S>(self, ser: impl ::alkahest::private::Into<S>) -> ::alkahest::private::Result<S::Ok, S::Error>
                            where
                                S: ::alkahest::private::Serializer
                            {
                                // Checks compilation of code in the block.
                                #[allow(unused)]
                                let _ = || {
                                    #(let _: [(); #field_ids_check] = <#formula_type>::#field_names_check();)*
                                    #field_count_check
                                };

                                let mut ser = ser.into();
                                #(
                                    let with_formula = ::alkahest::private::with_formula(|s: &#formula_type| &s.#field_names);
                                    with_formula.write_value(&mut ser, &self.#field_names)?;
                                )*
                                ser.finish()
                            }
                        }

                        impl #impl_generics ::alkahest::private::Serialize<#formula_type> for &#ident #type_generics #where_clause {
                            fn serialize<S>(self, ser: impl ::alkahest::private::Into<S>) -> ::alkahest::private::Result<S::Ok, S::Error>
                            where
                                S: ::alkahest::private::Serializer
                            {
                                let mut ser = ser.into();
                                #(
                                    let with_formula = ::alkahest::private::with_formula(|s: &#formula_type| &s.#field_names);
                                    with_formula.write_value(&mut ser, &self.#field_names)?;
                                )*
                                ser.finish()
                            }
                        }
                    })
                }
                (reference, Some(owned)) => {
                    let formula_type = &owned.ty;
                    let field_count_check = if cfg.check_fields {
                        quote::quote! {
                            let _: [(); #field_count] = <#formula_type>::__alkahest_formula_field_count();
                        }
                    } else {
                        quote::quote! {}
                    };

                    let mut generics = input.generics.clone();

                    generics.lt_token = generics.lt_token.or(owned.generics.lt_token);
                    generics.gt_token = generics.gt_token.or(owned.generics.gt_token);
                    generics.params.extend(owned.generics.params.into_iter());

                    if let Some(where_clause) = owned.generics.where_clause {
                        generics
                            .make_where_clause()
                            .predicates
                            .extend(where_clause.predicates);
                    }

                    let (impl_generics, _type_generics, where_clause) = generics.split_for_impl();

                    let mut tokens = quote::quote! {
                        impl #impl_generics ::alkahest::private::Serialize<#formula_type> for #ident #type_generics #where_clause {
                            fn serialize<S>(self, ser: impl ::alkahest::private::Into<S>) -> ::alkahest::private::Result<S::Ok, S::Error>
                            where
                                S: ::alkahest::private::Serializer
                            {
                                // Checks compilation of code in the block.
                                #[allow(unused)]
                                let _ = || {
                                    #(let _: [(); #field_ids_check] = <#formula_type>::#field_names_check();)*
                                    #field_count_check
                                };

                                let mut ser = ser.into();
                                #(
                                    let with_formula = ::alkahest::private::with_formula(|s: &#formula_type| &s.#field_names);
                                    with_formula.write_value(&mut ser, self.#field_names)?;
                                )*
                                ser.finish()
                            }
                        }
                    };

                    if let Some(reference) = reference {
                        let formula_type = &reference.ty;
                        generics = input.generics.clone();

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

                        let (impl_generics, _type_generics, where_clause) =
                            generics.split_for_impl();

                        tokens.extend(quote::quote! {
                            impl #impl_generics ::alkahest::private::Serialize<#formula_type> for &#ident #type_generics #where_clause {
                                fn serialize<S>(self, ser: impl ::alkahest::private::Into<S>) -> ::alkahest::private::Result<S::Ok, S::Error>
                                where
                                    S: ::alkahest::private::Serializer
                                {
                                    let mut ser = ser.into();
                                    #(
                                        let with_formula = ::alkahest::private::with_formula(|s: &#formula_type| &s.#field_names);
                                        with_formula.write_value(&mut ser, &self.#field_names)?;
                                    )*
                                    ser.finish()
                                }
                            }
                        });
                    }

                    Ok(tokens)
                }
            }
        }
        syn::Data::Enum(data) => {
            let cfg = Config::for_enum(args, &data, ident, generics);

            if let Some(variant) = &cfg.variant {
                return Err(syn::Error::new_spanned(
                    variant,
                    "Variant can be specified only for structs",
                ));
            }

            let field_names_checks: Vec<_> = data
                .variants
                .iter()
                .flat_map(|v| match (cfg.check_fields, &v.fields) {
                    (true, syn::Fields::Named(_)) => v
                        .fields
                        .iter()
                        .map(|field| {
                            quote::format_ident!(
                                "__alkahest_formula_variant_{}_field_{}_idx_is",
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
                            "__alkahest_formula_variant_{}_field_count",
                            variant.ident,
                        )
                    })
                    .collect();

            let field_counts: Vec<_> = data.variants.iter().map(|v| v.fields.len()).collect();

            let variant_names = data.variants.iter().map(|v| &v.ident).collect::<Vec<_>>();

            let bound_field_names = data
                .variants
                .iter()
                .map(|v| {
                    v.fields
                        .iter()
                        .enumerate()
                        .map(|(index, field)| match &field.ident {
                            Some(ident) => ident.clone(),
                            None => quote::format_ident!("_{}", index),
                        })
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();

            let field_bind_names = data
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
                            .map(|(index, _)| quote::format_ident!("_{}", index));

                        quote::quote! {
                            ( #(#names),* )
                        }
                    }
                    syn::Fields::Unit => quote::quote! {},
                })
                .collect::<Vec<_>>();

            let field_variant_name_ids: Vec<syn::Ident> = data
                .variants
                .iter()
                .map(|variant| {
                    quote::format_ident!("__alkahest_formula_variant_{}_idx", variant.ident,)
                })
                .collect();

            match (cfg.reference, cfg.owned) {
                (None, None) => unreachable!(),
                (Some(reference), None) => {
                    let formula_type = &reference.ty;
                    let field_count_check = if cfg.check_fields {
                        quote::quote! {
                            #(let _: [(); #field_counts] = <#formula_type>::#field_count_checks();)*
                        }
                    } else {
                        quote::quote! {}
                    };

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
                    Ok(quote::quote! {
                        impl #impl_generics ::alkahest::private::Serialize<#formula_type> for #ident #type_generics #where_clause {
                            fn serialize<S>(self, ser: impl ::alkahest::private::Into<S>) -> ::alkahest::private::Result<S::Ok, S::Error>
                            where
                                S: ::alkahest::private::Serializer
                            {
                                // Checks compilation of code in the block.
                                #[allow(unused)]
                                let _ = || {
                                    #(let _: [(); #field_ids_checks] = <#formula_type>::#field_names_checks();)*
                                    #field_count_check
                                };

                                let mut ser = ser.into();
                                match self {
                                    #(
                                        #ident::#variant_names #field_bind_names => {
                                            ser.write_value::<u32, u32>(<#formula_type>::#field_variant_name_ids())?;
                                            #(
                                                let with_formula = ::alkahest::private::with_formula(|s: &#formula_type| match s {
                                                    #formula_type::#variant_names { #field_bind_names } => #bound_field_names,
                                                    _ => unreachable!(),
                                                });
                                                with_formula.write_value(&mut ser, #bound_field_names)?;
                                            )*
                                        }
                                    )*
                                }
                                ser.finish()
                            }
                        }

                        impl #impl_generics ::alkahest::private::Serialize<#formula_type> for &#ident #type_generics #where_clause {
                            fn serialize<S>(self, ser: impl ::alkahest::private::Into<S>) -> ::alkahest::private::Result<S::Ok, S::Error>
                            where
                                S: ::alkahest::private::Serializer
                            {
                                let mut ser = ser.into();
                                match self {
                                    #(
                                        #ident::#variant_names #field_bind_names => {
                                            ser.write_value::<u32, u32>(<#formula_type>::#field_variant_name_ids())?;
                                            #(
                                                let with_formula = ::alkahest::private::with_formula(|s: &#formula_type| match s {
                                                    #formula_type::#variant_names { #field_bind_names } => #bound_field_names,
                                                    _ => unreachable!(),
                                                });
                                                with_formula.write_value(&mut ser, #bound_field_names)?;
                                            )*
                                        }
                                    )*
                                }
                                ser.finish()
                            }
                        }
                    })
                }
                (reference, Some(owned)) => {
                    let formula_type = &owned.ty;
                    let field_count_check = if cfg.check_fields {
                        quote::quote! {
                            #(let _: [(); #field_counts] = <#formula_type>::#field_count_checks();)*
                        }
                    } else {
                        quote::quote! {}
                    };

                    let mut generics = input.generics.clone();

                    generics.lt_token = generics.lt_token.or(owned.generics.lt_token);
                    generics.gt_token = generics.gt_token.or(owned.generics.gt_token);
                    generics.params.extend(owned.generics.params.into_iter());

                    if let Some(where_clause) = owned.generics.where_clause {
                        generics
                            .make_where_clause()
                            .predicates
                            .extend(where_clause.predicates);
                    }

                    let (impl_generics, _type_generics, where_clause) = generics.split_for_impl();

                    let mut tokens = quote::quote! {
                        impl #impl_generics ::alkahest::private::Serialize<#formula_type> for #ident #type_generics #where_clause {
                            fn serialize<S>(self, ser: impl ::alkahest::private::Into<S>) -> ::alkahest::private::Result<S::Ok, S::Error>
                            where
                                S: ::alkahest::private::Serializer
                            {
                                // Checks compilation of code in the block.
                                #[allow(unused)]
                                let _ = || {
                                    #(let _: [(); #field_ids_checks] = <#formula_type>::#field_names_checks();)*
                                    #field_count_check
                                };

                                let mut ser = ser.into();
                                match self {
                                    #(
                                        #ident::#variant_names #field_bind_names => {
                                            ser.write_value::<u32, u32>(<#formula_type>::#field_variant_name_ids())?;
                                            #(
                                                let with_formula = ::alkahest::private::with_formula(|s: &#formula_type| match s {
                                                    #formula_type::#variant_names { #field_bind_names } => #bound_field_names,
                                                    _ => unreachable!(),
                                                });
                                                with_formula.write_value(&mut ser, #bound_field_names)?;
                                            )*
                                        }
                                    )*
                                }
                                ser.finish()
                            }
                        }
                    };

                    if let Some(reference) = reference {
                        let formula_type = &reference.ty;
                        generics = input.generics.clone();

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

                        let (impl_generics, _type_generics, where_clause) =
                            generics.split_for_impl();

                        tokens.extend(quote::quote! {
                            impl #impl_generics ::alkahest::private::Serialize<#formula_type> for &#ident #type_generics #where_clause {
                                fn serialize<S>(self, ser: impl ::alkahest::private::Into<S>) -> ::alkahest::private::Result<S::Ok, S::Error>
                                where
                                    S: ::alkahest::private::Serializer
                                {
                                    let mut ser = ser.into();
                                    match self {
                                        #(
                                            #ident::#variant_names #field_bind_names => {
                                                ser.write_value::<u32, u32>(<#formula_type>::#field_variant_name_ids())?;
                                                #(
                                                    let with_formula = ::alkahest::private::with_formula(|s: &#formula_type| match s {
                                                        #formula_type::#variant_names { #field_bind_names } => #bound_field_names,
                                                        _ => unreachable!(),
                                                    });
                                                    with_formula.write_value(&mut ser, #bound_field_names)?;
                                                )*
                                            }
                                        )*
                                    }
                                    ser.finish()
                                }
                            }
                        });
                    }

                    Ok(tokens)
                }
            }
        }
    }
}
