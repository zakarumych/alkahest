use proc_macro2::TokenStream;

use crate::attrs::{parse_attributes, Args, Formula};

struct Config {
    formula: Formula,

    /// Signals if fields should be checked to match on formula.
    /// `false` if `formula` is inferred to `Self`.
    check_fields: bool,

    /// Signals that it can deserialize
    /// formulas with appended fields.
    /// This requires that last field is `SizedFormula`
    non_exhaustive: bool,

    /// Deserializer lifetime
    de: syn::Lifetime,
}

impl Config {
    fn for_struct(args: Args, data: &syn::DataStruct) -> syn::Result<Self> {
        let non_exhaustive = args.non_exhaustive.is_some();
        match args.deserialize.or(args.common) {
            None => {
                let de: syn::LifetimeDef = syn::parse_quote!('de);

                // Add predicates that fields implement
                // `SizedFormula + Deserialize<'de, #field_type>`
                // Except that last one if `non_exhaustive` is not set.
                let predicates = data.fields.iter().map(|field| -> syn::WherePredicate {
                        let ty = &field.ty;
                        syn::parse_quote! { #ty: ::alkahest::private::Formula + ::alkahest::private::Deserialize<'de, #ty> }
                    }).collect();

                // Add `'de` generic parameter
                let generics = syn::Generics {
                    lt_token: Some(Default::default()),
                    params: std::iter::once(syn::GenericParam::Lifetime(de.clone())).collect(),
                    gt_token: Some(Default::default()),
                    where_clause: Some(syn::WhereClause {
                        where_token: Default::default(),
                        predicates,
                    }),
                };

                Ok(Config {
                    formula: Formula {
                        path: syn::parse_quote!(Self),
                        generics,
                    },
                    check_fields: false,
                    non_exhaustive,
                    de: de.lifetime,
                })
            }
            Some(mut formula) => {
                let de: syn::LifetimeDef;

                // If no parameters specified, add `'de` parameter
                if formula.generics.params.is_empty() {
                    de = syn::parse_quote!('de);
                    formula.generics.params.push(de.clone().into());
                } else {
                    let first = formula.generics.params.first().unwrap().clone();

                    de = match first {
                        syn::GenericParam::Lifetime(lt) => lt,
                        param => {
                            return Err(syn::Error::new_spanned(
                                param,
                                "First parameter must be deserializer's lifetime",
                            ));
                        }
                    };
                }

                Ok(Config {
                    formula,
                    check_fields: true,
                    non_exhaustive,
                    de: de.lifetime,
                })
            }
        }
    }

    fn for_enum(args: Args, data: &syn::DataEnum) -> syn::Result<Self> {
        let non_exhaustive = args.non_exhaustive.is_some();
        match args.deserialize.or(args.common) {
            None => {
                let de: syn::LifetimeDef = syn::parse_quote!('de);

                // Add predicates that fields implement
                // `SizedFormula + Deserialize<'de, #field_type>`
                // Except that last one if `non_exhaustive` is not set.
                let predicates = data.variants.iter().flat_map(|v| v.fields.iter().map(|field| -> syn::WherePredicate {
                        let ty = &field.ty;
                        syn::parse_quote! { #ty: ::alkahest::private::Formula + ::alkahest::private::Deserialize<'de, #ty> }
                    })).collect();

                // Add `'de` generic parameter
                let generics = syn::Generics {
                    lt_token: Some(Default::default()),
                    params: std::iter::once(syn::GenericParam::Lifetime(de.clone())).collect(),
                    gt_token: Some(Default::default()),
                    where_clause: Some(syn::WhereClause {
                        where_token: Default::default(),
                        predicates,
                    }),
                };

                Ok(Config {
                    formula: Formula {
                        path: syn::parse_quote!(Self),
                        generics,
                    },
                    check_fields: false,
                    non_exhaustive,
                    de: de.lifetime,
                })
            }
            Some(mut formula) => {
                let de: syn::LifetimeDef;

                // If no parameters specified, add `'de` parameter
                if formula.generics.params.is_empty() {
                    de = syn::parse_quote!('de);
                    formula.generics.params.push(de.clone().into());
                } else {
                    let first = formula.generics.params.first().unwrap().clone();

                    de = match first {
                        syn::GenericParam::Lifetime(lt) => lt,
                        param => {
                            return Err(syn::Error::new_spanned(
                                param,
                                "First parameter must be deserializer's lifetime",
                            ));
                        }
                    };
                }

                Ok(Config {
                    formula,
                    check_fields: true,
                    non_exhaustive,
                    de: de.lifetime,
                })
            }
        }
    }
}

pub fn derive(input: proc_macro::TokenStream) -> syn::Result<TokenStream> {
    let no_named_fields = syn::punctuated::Punctuated::<syn::Field, syn::Token![,]>::new();

    let input = syn::parse::<syn::DeriveInput>(input)?;
    let args = parse_attributes(&input.attrs)?;

    let ident = &input.ident;

    match input.data {
        syn::Data::Union(_) => Err(syn::Error::new_spanned(
            input,
            "Deserialize cannot be derived for unions",
        )),
        syn::Data::Struct(data) => {
            let Config {
                formula,
                check_fields,
                non_exhaustive,
                de,
            } = Config::for_struct(args, &data)?;

            let formula_path = &formula.path;

            let mut deserialize_generics = input.generics.clone();

            deserialize_generics.lt_token =
                deserialize_generics.lt_token.or(formula.generics.lt_token);
            deserialize_generics.gt_token =
                deserialize_generics.gt_token.or(formula.generics.gt_token);
            deserialize_generics
                .params
                .extend(formula.generics.params.into_iter());

            if let Some(where_clause) = formula.generics.where_clause {
                deserialize_generics
                    .make_where_clause()
                    .predicates
                    .extend(where_clause.predicates);
            }

            let field_names_order_checks = match (check_fields, &data.fields) {
                (true, syn::Fields::Named(fields)) => fields
                    .named
                    .iter()
                    .map(|field| {
                        quote::format_ident!(
                            "__ALKAHEST_FORMULA_FIELD_{}_IDX",
                            field.ident.as_ref().unwrap(),
                        )
                    })
                    .collect(),
                _ => Vec::new(),
            };

            let field_ids_checks = match (check_fields, &data.fields) {
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

            let consume_tail;

            if non_exhaustive {
                consume_tail = quote::quote! {
                    des.read_all_bytes();
                };
            } else {
                consume_tail = quote::quote! {};
            }

            let field_count = data.fields.len();
            let field_count_check = if check_fields && !non_exhaustive {
                quote::quote! {
                    let _: [(); #field_count] = #formula_path::__ALKAHEST_FORMULA_FIELD_COUNT;
                }
            } else {
                quote::quote! {}
            };

            let (_impl_generics, type_generics, _where_clause) = input.generics.split_for_impl();
            let (impl_deserialize_generics, _type_deserialize_generics, where_serialize_clause) =
                deserialize_generics.split_for_impl();
            Ok(quote::quote! {
                impl #impl_deserialize_generics ::alkahest::private::Deserialize<#de, #formula_path> for #ident #type_generics #where_serialize_clause {
                    #[inline(always)]
                    fn deserialize(mut de: ::alkahest::private::Deserializer<#de>) -> ::alkahest::private::Result<Self, ::alkahest::private::Error> {
                        // Checks compilation of code in the block.
                        #[allow(unused)]
                        {
                            #(let _: [(); #field_ids_checks] = #formula_path::#field_names_order_checks;)*
                            #field_count_check
                        }

                        #(
                            let with_formula = ::alkahest::private::with_formula(|s: &#formula_path| match *s {
                                #formula_path #bind_ref_names => #bound_names,
                                _ => unreachable!(),
                            });
                            let #bound_names = with_formula.read_value(&mut de)?;
                        )*
                        #consume_tail
                        de.finish()?;

                        let value = #ident #bind_names;
                        ::alkahest::private::Result::Ok(value)
                    }

                    #[inline(always)]
                    fn deserialize_in_place(&mut self, mut de: ::alkahest::private::Deserializer<#de>) -> Result<(), ::alkahest::private::Error> {
                        let #ident #bind_ref_mut_names = *self;

                        #(
                            let with_formula = ::alkahest::private::with_formula(|s: &#formula_path| match *s {
                                #formula_path #bind_ref_names => #bound_names,
                                _ => unreachable!(),
                            });
                            with_formula.read_in_place(#bound_names, &mut de)?;
                        )*
                        #consume_tail
                        de.finish()
                    }
                }
            })
        }
        syn::Data::Enum(data) => {
            let Config {
                formula,
                check_fields,
                non_exhaustive,
                de,
            } = Config::for_enum(args, &data)?;
            let formula_path = &formula.path;

            let mut deserialize_generics = input.generics.clone();

            deserialize_generics.lt_token =
                deserialize_generics.lt_token.or(formula.generics.lt_token);
            deserialize_generics.gt_token =
                deserialize_generics.gt_token.or(formula.generics.gt_token);
            deserialize_generics
                .params
                .extend(formula.generics.params.into_iter());

            if let Some(where_clause) = formula.generics.where_clause {
                deserialize_generics
                    .make_where_clause()
                    .predicates
                    .extend(where_clause.predicates);
            }

            let field_names_order_checks = match check_fields {
                false => Vec::new(),
                true => data
                    .variants
                    .iter()
                    .flat_map(|v| {
                        match &v.fields {
                            syn::Fields::Named(fields) => fields.named.iter(),
                            _ => no_named_fields.iter(),
                        }
                        .map(move |field| {
                            quote::format_ident!(
                                "__ALKAHEST_FORMULA_VARIANT_{}_FIELD_{}_IDX",
                                v.ident,
                                field.ident.as_ref().unwrap(),
                            )
                        })
                    })
                    .collect(),
            };

            let field_ids_checks = match check_fields {
                false => Vec::new(),
                true => data
                    .variants
                    .iter()
                    .flat_map(|v| match &v.fields {
                        syn::Fields::Named(fields) => 0..fields.named.len(),
                        _ => 0..0,
                    })
                    .collect(),
            };

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

            let consume_tail;

            if non_exhaustive {
                consume_tail = quote::quote! {
                    des.read_all_bytes();
                };
            } else {
                consume_tail = quote::quote! {};
            }

            let variant_count = data.variants.len();
            let variant_count_check = match check_fields {
                false => quote::quote! {},
                true => quote::quote! {
                    let _: [(); #variant_count] = #formula_path::__ALKAHEST_FORMULA_VARIANT_COUNT;
                },
            };

            let field_counts: Vec<_> = data.variants.iter().map(|v| v.fields.len()).collect();
            let field_count_check = if check_fields && !non_exhaustive {
                quote::quote! {
                    #(let _: [(); #field_counts] = #formula_path::#field_count_checks;)*
                }
            } else {
                quote::quote! {}
            };

            let (_impl_generics, type_generics, _where_clause) = input.generics.split_for_impl();
            let (impl_deserialize_generics, _type_deserialize_generics, where_serialize_clause) =
                deserialize_generics.split_for_impl();
            Ok(quote::quote! {
                impl #impl_deserialize_generics ::alkahest::private::Deserialize<#de, #formula_path> for #ident #type_generics #where_serialize_clause {
                    #[inline(always)]
                    fn deserialize(mut de: ::alkahest::private::Deserializer<#de>) -> ::alkahest::private::Result<Self, ::alkahest::private::Error> {
                        // Checks compilation of code in the block.
                        #[allow(unused)]
                        {
                            #(let _: [(); #field_ids_checks] = #formula_path::#field_names_order_checks;)*
                            #field_count_check
                            #variant_count_check
                        }

                        let variant_idx = de.read_auto::<::alkahest::private::u32>()?;
                        match variant_idx {
                            #(
                                #formula_path::#variant_name_ids => {
                                    #(
                                        let with_formula = ::alkahest::private::with_formula(|s: &#formula_path| match *s {
                                            #[allow(unused_variables)]
                                            #formula_path::#variant_names #bind_ref_names => #bound_names,
                                            _ => unreachable!(),
                                        });
                                        let #bound_names = with_formula.read_value(&mut de)?;
                                    )*
                                    #consume_tail
                                    de.finish()?;
                                    ::alkahest::private::Result::Ok(#ident::#variant_names #bind_names)
                                }
                            )*
                            invalid => ::alkahest::private::err(::alkahest::private::Error::WrongVariant(invalid)),
                        }
                    }

                    #[inline(always)]
                    fn deserialize_in_place(&mut self, mut de: ::alkahest::private::Deserializer<#de>) -> Result<(), ::alkahest::private::Error> {
                        // Checks compilation of code in the block.
                        #[allow(unused)]
                        let _ = || {
                            #(let _: [(); #field_ids_checks] = #formula_path::#field_names_order_checks;)*
                        };
                        #field_count_check

                        let variant_idx = de.read_auto::<::alkahest::private::u32>()?;
                        match (variant_idx, self) {
                            #(
                                (#formula_path::#variant_name_ids, #ident::#variant_names #bind_ref_mut_names) => {
                                    #(
                                        let with_formula = ::alkahest::private::with_formula(|s: &#formula_path| match *s {
                                            #[allow(unused_variables)]
                                            #formula_path::#variant_names #bind_ref_names => #bound_names,
                                            _ => unreachable!(),
                                        });
                                        with_formula.read_in_place(#bound_names, &mut de)?;
                                    )*
                                    #consume_tail
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
                                        let #bound_names = with_formula.read_value(&mut de)?;
                                    )*
                                    #consume_tail
                                    de.finish()?;
                                    *me = #ident::#variant_names #bind_names;
                                    ::alkahest::private::Result::Ok(())
                                }
                            )*
                            (invalid, _) => ::alkahest::private::err(::alkahest::private::Error::WrongVariant(invalid)),
                        }
                    }
                }
            })
        }
    }
}
