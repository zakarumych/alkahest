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
                        lt_token: Some(Default::default()),
                        params: syn::parse_quote!('ser),
                        gt_token: Some(Default::default()),
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
            (None, Some(None)) => {
                // Add predicates that fields implement
                // `T: Formula` and `T: Serialize<T>`
                let predicates = data
                    .fields
                    .iter()
                    .map(|field| -> syn::WherePredicate {
                        let ty = &field.ty;
                        syn::parse_quote! { #ty: ::alkahest::Formula }
                    })
                    .chain(data.fields.iter().map(|field| -> syn::WherePredicate {
                        let ty = &field.ty;
                        syn::parse_quote! { #ty: ::alkahest::Serialize<#ty> }
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
            (None, None) => {
                // Add predicates that fields implement
                // `T: Formula` and `&T: Serialize<T>`
                let predicates = data
                    .fields
                    .iter()
                    .map(|field| -> syn::WherePredicate {
                        let ty = &field.ty;
                        syn::parse_quote! { #ty: ::alkahest::Formula }
                    })
                    .chain(data.fields.iter().map(|field| -> syn::WherePredicate {
                        let ty = &field.ty;
                        syn::parse_quote! { &'ser #ty: ::alkahest::Serialize<#ty> }
                    }))
                    .collect();

                let generics = syn::Generics {
                    lt_token: Some(Default::default()),
                    params: syn::parse_quote!('ser),
                    gt_token: Some(Default::default()),
                    where_clause: Some(syn::WhereClause {
                        where_token: Default::default(),
                        predicates,
                    }),
                };

                let reference = Formula {
                    ty: syn::parse_quote!(#ident #type_generics),
                    generics,
                };

                // Add predicates that fields implement
                // `T: Formula` and `T: Serialize<T>`
                let predicates = data
                    .fields
                    .iter()
                    .map(|field| -> syn::WherePredicate {
                        let ty = &field.ty;
                        syn::parse_quote! { #ty: ::alkahest::Formula }
                    })
                    .chain(data.fields.iter().map(|field| -> syn::WherePredicate {
                        let ty = &field.ty;
                        syn::parse_quote! { #ty: ::alkahest::Serialize<#ty> }
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
            (None, Some(Some(owned))) => Config {
                owned: Some(owned),
                reference: None,
                variant: args.variant,
                check_fields: true,
            },
            (Some(mut reference), None | Some(None)) => {
                if reference.generics.params.is_empty() {
                    reference.generics.lt_token = Some(Default::default());
                    reference.generics.gt_token = Some(Default::default());
                    reference
                        .generics
                        .params
                        .push(syn::GenericParam::Lifetime(syn::parse_quote!('ser)));
                }
                Config {
                    reference: Some(reference.clone()),
                    owned: None,
                    variant: args.variant,
                    check_fields: true,
                }
            }
            (Some(mut reference), Some(Some(owned))) => {
                if reference.generics.params.is_empty() {
                    reference.generics.lt_token = Some(Default::default());
                    reference.generics.gt_token = Some(Default::default());
                    reference
                        .generics
                        .params
                        .push(syn::GenericParam::Lifetime(syn::parse_quote!('ser)));
                }
                Config {
                    reference: Some(reference),
                    owned: Some(owned),
                    variant: args.variant,
                    check_fields: true,
                }
            }
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

            let field_check_names = match (cfg.check_fields, &data.fields) {
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

            let field_check_ids = match (cfg.check_fields, &data.fields) {
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
                    let check_field_count = if cfg.check_fields {
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
                        impl #impl_generics ::alkahest::Serialize<#formula_type> for &'ser #ident #type_generics #where_clause {
                            fn serialize<S>(self, ser: impl ::alkahest::private::Into<S>) -> ::alkahest::private::Result<S::Ok, S::Error>
                            where
                                S: ::alkahest::Serializer
                            {
                                // Checks compilation of code in the block.
                                #[allow(unused)]
                                let _ = || {
                                    #(let _: [(); #field_check_ids] = <#formula_type>::#field_check_names();)*
                                    #check_field_count
                                };

                                let mut ser = ser.into();
                                #(
                                    let with_formula = ::alkahest::private::with_formula(|s: &#formula_type| &s.#field_names);
                                    with_formula.write_value(&mut ser, &self.#field_names)?;
                                )*
                                ser.finish()
                            }
                        }

                        impl #impl_generics ::alkahest::Serialize<#formula_type> for #ident #type_generics #where_clause {
                            fn serialize<S>(self, ser: impl ::alkahest::private::Into<S>) -> ::alkahest::private::Result<S::Ok, S::Error>
                            where
                                S: ::alkahest::Serializer
                            {
                                let mut ser = ser.into();
                                #(
                                    let with_formula = ::alkahest::private::with_formula(|s: &#formula_type| &s.#field_names);
                                    with_formula.write_value(&mut ser, self.#field_names)?;
                                )*
                                ser.finish()
                            }
                        }
                    })
                }
                (reference, Some(owned)) => {
                    let formula_type = &owned.ty;
                    let check_field_count = if cfg.check_fields {
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
                        impl #impl_generics ::alkahest::Serialize<#formula_type> for #ident #type_generics #where_clause {
                            fn serialize<S>(self, ser: impl ::alkahest::private::Into<S>) -> ::alkahest::private::Result<S::Ok, S::Error>
                            where
                                S: ::alkahest::Serializer
                            {
                                // Checks compilation of code in the block.
                                #[allow(unused)]
                                let _ = || {
                                    #(let _: [(); #field_check_ids] = <#formula_type>::#field_check_names();)*
                                    #check_field_count
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
                            impl #impl_generics ::alkahest::Serialize<#formula_type> for &'ser #ident #type_generics #where_clause {
                                fn serialize<S>(self, ser: impl ::alkahest::private::Into<S>) -> ::alkahest::private::Result<S::Ok, S::Error>
                                where
                                    S: ::alkahest::Serializer
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
        syn::Data::Enum(_) => {
            todo!()
        }
    }
}
