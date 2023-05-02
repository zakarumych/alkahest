use std::collections::HashSet;

use proc_macro2::TokenStream;

use crate::{
    attrs::SerializeArgs, enum_field_order_checks, filter_type_param, is_generic_ty,
    struct_field_order_checks,
};

struct Config {
    formula: syn::Path,
    generics: syn::Generics,

    variant: Option<syn::Ident>,

    /// Signals if fields should be checked to match on formula.
    /// `false` if `formula` is inferred to `Self`.
    check_fields: bool,
}

impl Config {
    #[allow(clippy::too_many_lines)]
    fn for_type(
        args: SerializeArgs,
        data: &syn::Data,
        generics: &syn::Generics,
        by_ref: bool,
    ) -> Self {
        let params = &generics.params;

        match (args.formula, args.generics) {
            (None, None) if params.is_empty() => Config {
                formula: syn::parse_quote! { Self },
                generics: syn::Generics::default(),
                variant: None,
                check_fields: false,
            },
            (None, None) => {
                let mut generics = syn::Generics {
                    lt_token: None,
                    params: syn::punctuated::Punctuated::default(),
                    gt_token: None,
                    where_clause: None,
                };

                // Add predicates that fields implement
                // `T: Formula + Serialize<T>`
                // for fields where generics are involved.

                match data {
                    syn::Data::Union(_) => unreachable!(),
                    syn::Data::Struct(data) => {
                        let mut all_generic_field_types: HashSet<_> =
                            data.fields.iter().map(|f| &f.ty).collect();
                        all_generic_field_types
                            .retain(|ty| is_generic_ty(ty, &filter_type_param(params.iter())));

                        if !all_generic_field_types.is_empty() {
                            if by_ref {
                                let predicates = all_generic_field_types.iter().map(|ty| -> syn::WherePredicate {
                                    syn::parse_quote! { #ty: ::alkahest::private::Formula }
                                }).chain(all_generic_field_types.iter().map(|ty| -> syn::WherePredicate {
                                    syn::parse_quote! { for<'ser> &'ser #ty: ::alkahest::private::Serialize<#ty> }
                                }));
                                generics.make_where_clause().predicates.extend(predicates);
                            } else {
                                let predicates = all_generic_field_types.iter().map(|ty| -> syn::WherePredicate {
                                    syn::parse_quote! { #ty: ::alkahest::private::Formula + ::alkahest::private::Serialize<#ty> }
                                });
                                generics.make_where_clause().predicates.extend(predicates);
                            }
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
                            if by_ref {
                                let predicates = all_generic_field_types.iter().map(|ty| -> syn::WherePredicate {
                                    syn::parse_quote! { #ty: ::alkahest::private::Formula }
                                }).chain(all_generic_field_types.iter().map(|ty| -> syn::WherePredicate {
                                    syn::parse_quote! { for<'ser> &'ser #ty: ::alkahest::private::Serialize<#ty> }
                                }));
                                generics.make_where_clause().predicates.extend(predicates);
                            } else {
                                let predicates = all_generic_field_types.iter().map(|ty| -> syn::WherePredicate {
                                    syn::parse_quote! { #ty: ::alkahest::private::Formula + ::alkahest::private::Serialize<#ty> }
                                });
                                generics.make_where_clause().predicates.extend(predicates);
                            }
                        }
                    }
                }

                Config {
                    formula: syn::parse_quote! { Self },
                    generics,
                    variant: args.variant,
                    check_fields: false,
                }
            }
            (None, Some(generics)) => Config {
                formula: syn::parse_quote! { Self },
                generics,
                variant: args.variant,
                check_fields: true,
            },
            (Some(formula), None) => Config {
                formula,
                generics: syn::Generics::default(),
                variant: args.variant,
                check_fields: false,
            },
            (Some(formula), Some(generics)) => Config {
                formula,
                generics,
                variant: args.variant,
                check_fields: true,
            },
        }
    }
}

#[allow(clippy::too_many_lines)]
pub fn derive(
    args: SerializeArgs,
    input: &syn::DeriveInput,
    by_ref: bool,
) -> syn::Result<TokenStream> {
    let ident = &input.ident;
    let generics = &input.generics;
    let (_impl_generics, type_generics, _where_clause) = generics.split_for_impl();

    let cfg = Config::for_type(args, &input.data, generics, by_ref);

    match &input.data {
        syn::Data::Union(_) => Err(syn::Error::new_spanned(
            input,
            "Serialize cannot be derived for unions",
        )),
        syn::Data::Struct(data) => {
            let field_checks = if cfg.check_fields {
                struct_field_order_checks(&data, cfg.variant.as_ref(), &input.ident, &cfg.formula)
            } else {
                TokenStream::new()
            };

            let field_count = data.fields.len();

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

            let with_variant = match &cfg.variant {
                None => quote::quote! {},
                Some(v) => quote::quote! { :: #v },
            };

            let start_stack_size = match &cfg.variant {
                None => quote::quote! { 0usize },
                Some(_) => quote::quote! { ::alkahest::private::VARIANT_SIZE },
            };

            let formula_path = &cfg.formula;

            let write_variant = match &cfg.variant {
                None => quote::quote! {},
                Some(v) => {
                    let variant_name_idx =
                        quote::format_ident!("__ALKAHEST_FORMULA_VARIANT_{}_IDX", v);
                    quote::quote! { ::alkahest::private::write_exact_size_field::<u32, u32, _>(#formula_path::#variant_name_idx, __sizes, __buffer.reborrow())?; }
                }
            };

            let mut generics = input.generics.clone();

            generics.lt_token = generics.lt_token.or(cfg.generics.lt_token);
            generics.gt_token = generics.gt_token.or(cfg.generics.gt_token);
            generics.params.extend(cfg.generics.params.into_iter());

            if let Some(where_clause) = cfg.generics.where_clause {
                generics
                    .make_where_clause()
                    .predicates
                    .extend(where_clause.predicates);
            }

            let (impl_generics, _type_generics, where_clause) = generics.split_for_impl();

            let tokens = if by_ref {
                quote::quote! {
                    impl #impl_generics ::alkahest::private::SerializeRef<#formula_path> for #ident #type_generics #where_clause {
                        #[inline(always)]
                        fn serialize<__alkahest_Buffer>(&self, __sizes: &mut ::alkahest::private::Sizes, mut __buffer: __alkahest_Buffer) -> ::alkahest::private::Result<(), __alkahest_Buffer::Error>
                        where
                            __alkahest_Buffer: ::alkahest::private::Buffer,
                        {
                            #![allow(unused_mut)]
                            #field_checks

                            let #ident #bind_ref_names = *self;
                            #write_variant
                            #(
                                let with_formula = ::alkahest::private::with_formula(|s: &#formula_path| match *s {
                                    #formula_path #with_variant #bind_ref_names => #bound_names,
                                    _ => unreachable!(),
                                });
                                with_formula.write_field(#bound_names, __sizes, __buffer.reborrow(), #field_count == 1 + #field_ids)?;
                            )*
                            Ok(())
                        }

                        #[inline(always)]
                        fn size_hint(&self) -> ::alkahest::private::Option<::alkahest::private::Sizes> {
                            #![allow(unused_mut)]
                            #field_checks
                            if let ::alkahest::private::Option::Some(sizes) = ::alkahest::private::formula_fast_sizes::<#formula_path>() {
                                return Some(sizes);
                            }
                            let #ident #bind_ref_names = *self;
                            let mut __total = ::alkahest::private::Sizes::with_stack(#start_stack_size);
                            #(
                                let with_formula = ::alkahest::private::with_formula(|s: &#formula_path| match *s {
                                    #formula_path #with_variant #bind_ref_names => #bound_names,
                                    _ => unreachable!(),
                                });
                                __total += with_formula.size_hint(&#bound_names, #field_count == 1 + #field_ids)?;
                            )*
                            Some(__total)
                        }
                    }
                }
            } else {
                quote::quote! {
                    impl #impl_generics ::alkahest::private::Serialize<#formula_path> for #ident #type_generics #where_clause {
                        #[inline(always)]
                        fn serialize<__alkahest_Buffer>(self, __sizes: &mut ::alkahest::private::Sizes, mut __buffer: __alkahest_Buffer) -> ::alkahest::private::Result<(), __alkahest_Buffer::Error>
                        where
                            __alkahest_Buffer: ::alkahest::private::Buffer,
                        {
                            #![allow(unused_mut)]
                            #field_checks

                            let #ident #bind_names = self;
                            #write_variant
                            #(
                                let with_formula = ::alkahest::private::with_formula(|s: &#formula_path| match *s {
                                    #formula_path #with_variant #bind_ref_names => #bound_names,
                                    _ => unreachable!(),
                                });
                                with_formula.write_field(#bound_names, __sizes, __buffer.reborrow(), #field_count == 1 + #field_ids)?;
                            )*
                            Ok(())
                        }

                        #[inline(always)]
                        fn size_hint(&self) -> ::alkahest::private::Option<::alkahest::private::Sizes> {
                            #![allow(unused_mut)]
                            #field_checks
                            if let ::alkahest::private::Option::Some(sizes) = ::alkahest::private::formula_fast_sizes::<#formula_path>() {
                                return Some(sizes);
                            }
                            let #ident #bind_ref_names = *self;
                            let mut __total = ::alkahest::private::Sizes::with_stack(#start_stack_size);
                            #(
                                let with_formula = ::alkahest::private::with_formula(|s: &#formula_path| match *s {
                                    #formula_path #with_variant #bind_ref_names => #bound_names,
                                    _ => unreachable!(),
                                });
                                __total += with_formula.size_hint(#bound_names, #field_count == 1 + #field_ids)?;
                            )*
                            Some(__total)
                        }
                    }
                }
            };

            Ok(tokens)
        }
        syn::Data::Enum(data) => {
            let field_checks = if cfg.check_fields {
                enum_field_order_checks(&data, &input.ident, &cfg.formula)
            } else {
                TokenStream::new()
            };

            if let Some(variant) = &cfg.variant {
                return Err(syn::Error::new_spanned(
                    variant,
                    "Variant can be specified only for structs",
                ));
            }

            let field_ids: Vec<Vec<_>> = data
                .variants
                .iter()
                .map(|v| (0..v.fields.len()).collect())
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

            let formula_path = &cfg.formula;

            let mut generics = input.generics.clone();

            generics.lt_token = generics.lt_token.or(cfg.generics.lt_token);
            generics.gt_token = generics.gt_token.or(cfg.generics.gt_token);
            generics.params.extend(cfg.generics.params.into_iter());

            if let Some(where_clause) = cfg.generics.where_clause {
                generics
                    .make_where_clause()
                    .predicates
                    .extend(where_clause.predicates);
            }

            let (impl_generics, _type_generics, where_clause) = generics.split_for_impl();

            let tokens = if by_ref {
                quote::quote! {
                    impl #impl_generics ::alkahest::private::SerializeRef<#formula_path> for #ident #type_generics #where_clause {
                        #[inline(always)]
                        fn serialize<__alkahest_Buffer>(&self, __sizes: &mut ::alkahest::private::Sizes, mut __buffer: __alkahest_Buffer) -> ::alkahest::private::Result<(), __alkahest_Buffer::Error>
                        where
                            __alkahest_Buffer: ::alkahest::private::Buffer,
                        {
                            #![allow(unused_mut, unused_variables)]
                            #field_checks
                            match *self {
                                #(
                                    #ident::#variant_names #bind_ref_names => {
                                        ::alkahest::private::write_exact_size_field::<u32, u32, _>(#formula_path::#variant_name_ids, __sizes, __buffer.reborrow())?;
                                        #(
                                            let with_formula = ::alkahest::private::with_formula(|s: &#formula_path| match *s {
                                                #formula_path::#variant_names #bind_ref_names => #bound_names,
                                                _ => unreachable!(),
                                            });
                                            with_formula.write_field(#bound_names, __sizes, __buffer.reborrow(), #field_counts == 1 + #field_ids)?;
                                        )*
                                        Ok(())
                                    }
                                )*
                            }
                        }

                        #[inline(always)]
                        fn size_hint(&self) -> ::alkahest::private::Option<::alkahest::private::Sizes> {
                            #![allow(unused_mut, unused_variables)]
                            #field_checks
                            if let ::alkahest::private::Option::Some(size) = ::alkahest::private::formula_fast_sizes::<#formula_path>() {
                                return Some(size);
                            }
                            match *self {
                                #(
                                    #ident::#variant_names #bind_ref_names => {
                                        let mut __total = ::alkahest::private::Sizes::with_stack(::alkahest::private::VARIANT_SIZE);
                                        #(
                                            let with_formula = ::alkahest::private::with_formula(|s: &#formula_path| match *s {
                                                #formula_path::#variant_names #bind_ref_names => #bound_names,
                                                _ => unreachable!(),
                                            });
                                            __total += with_formula.size_hint(&#bound_names, #field_counts == 1 + #field_ids)?;
                                        )*
                                        Some(__total)
                                    }
                                )*
                            }
                        }
                    }
                }
            } else {
                quote::quote! {
                    impl #impl_generics ::alkahest::private::Serialize<#formula_path> for #ident #type_generics #where_clause {
                        #[inline(always)]
                        fn serialize<__alkahest_Buffer>(self, __sizes: &mut ::alkahest::private::Sizes, mut __buffer: __alkahest_Buffer) -> ::alkahest::private::Result<(), __alkahest_Buffer::Error>
                        where
                            __alkahest_Buffer: ::alkahest::private::Buffer,
                        {
                            #![allow(unused_mut, unused_variables)]
                            #field_checks
                            match self {
                                #(
                                    #ident::#variant_names #bind_names => {
                                        ::alkahest::private::write_exact_size_field::<u32, u32, _>(#formula_path::#variant_name_ids, __sizes, __buffer.reborrow())?;
                                        #(
                                            let with_formula = ::alkahest::private::with_formula(|s: &#formula_path| match *s {
                                                #formula_path::#variant_names #bind_ref_names => #bound_names,
                                                _ => unreachable!(),
                                            });
                                            with_formula.write_field(#bound_names, __sizes, __buffer.reborrow(), #field_counts == 1 + #field_ids)?;
                                        )*
                                        Ok(())
                                    }
                                )*
                            }
                        }

                        #[inline(always)]
                        fn size_hint(&self) -> ::alkahest::private::Option<::alkahest::private::Sizes> {
                            #![allow(unused_mut, unused_variables)]
                            #field_checks
                            if let ::alkahest::private::Option::Some(size) = ::alkahest::private::formula_fast_sizes::<#formula_path>() {
                                return Some(size);
                            }
                            match *self {
                                #(
                                    #ident::#variant_names #bind_ref_names => {
                                        let mut __total = ::alkahest::private::Sizes::with_stack(::alkahest::private::VARIANT_SIZE);
                                        #(
                                            let with_formula = ::alkahest::private::with_formula(|s: &#formula_path| match *s {
                                                #formula_path::#variant_names #bind_ref_names => #bound_names,
                                                _ => unreachable!(),
                                            });
                                            __total += with_formula.size_hint(#bound_names, #field_counts == 1 + #field_ids)?;
                                        )*
                                        Some(__total)
                                    }
                                )*
                            }
                        }
                    }
                }
            };

            Ok(tokens)
        }
    }
}
