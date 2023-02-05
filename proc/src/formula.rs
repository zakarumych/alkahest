use proc_macro2::TokenStream;
use syn::spanned::Spanned;

use crate::attrs::parse_attributes;

pub fn derive(input: proc_macro::TokenStream) -> syn::Result<TokenStream> {
    let input = syn::parse::<syn::DeriveInput>(input)?;
    let ident = &input.ident;

    let args = parse_attributes(&input.attrs)?;
    let non_exhaustive = args.non_exhaustive.is_some();

    if let Some(formula) = args
        .serialize
        .or(args.deserialize)
        .or(args.common)
        .or(args.owned.flatten())
    {
        return Err(syn::Error::new_spanned(
            formula.ty,
            "Formula type should not be specified for `Serialize` and `Deserialize` when type is also `Formula`",
        ));
    }

    if args.variant.is_some() {
        return Err(syn::Error::new_spanned(
            input,
            "Variant should not be specified for `Serialize` when type is also `Formula`",
        ));
    }

    match &input.data {
        syn::Data::Union(data) => {
            return Err(syn::Error::new_spanned(
                data.union_token,
                "Formula cannot be derived for unions",
            ))
        }
        syn::Data::Struct(data) => {
            let all_field_types: Vec<_> = data.fields.iter().map(|field| &field.ty).collect();

            let mut formula_generics = input.generics.clone();
            if !all_field_types.is_empty() && !input.generics.params.is_empty() {
                let predicates = all_field_types.iter().map(|ty| -> syn::WherePredicate {
                    syn::parse_quote_spanned! { ty.span() => #ty: ::alkahest::Formula }
                });

                let where_clause = formula_generics.make_where_clause();
                where_clause.predicates.extend(predicates);
            }

            let field_count = data.fields.len();

            let field_check_names = match &data.fields {
                syn::Fields::Named(fields) => fields
                    .named
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

            let field_check_ids = match &data.fields {
                syn::Fields::Named(fields) => (0..fields.named.len()).collect(),
                _ => Vec::new(),
            };

            let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

            let (formula_impl_generics, formula_type_generics, formula_where_clause) =
                formula_generics.split_for_impl();

            Ok(quote::quote! {
                impl #impl_generics #ident #type_generics #where_clause {
                    #(
                        #[doc(hidden)]
                        #[inline(always)]
                        pub const fn #field_check_names() -> [(); #field_check_ids] {
                            [(); #field_check_ids]
                        }
                    )*

                    #[doc(hidden)]
                    #[inline(always)]
                    pub const fn __alkahest_formula_field_count() -> [(); #field_count] {
                        [(); #field_count]
                    }
                }

                impl #formula_impl_generics ::alkahest::Formula for #ident #formula_type_generics #formula_where_clause {
                    const MAX_SIZE: ::alkahest::private::Option<::alkahest::private::usize> = {
                        #[allow(unused_mut)]
                        let mut max_size = Some(0);
                        #(
                            max_size = ::alkahest::private::combine_sizes(max_size, <#all_field_types as ::alkahest::Formula>::MAX_SIZE);
                        )*;
                        max_size
                    };
                }

                impl #formula_impl_generics ::alkahest::NonRefFormula for #ident #formula_type_generics #formula_where_clause {}
            })
        }
        syn::Data::Enum(data) => {
            let all_field_types: Vec<&syn::Type> = data
                .variants
                .iter()
                .flat_map(|variant| variant.fields.iter().map(|field| &field.ty))
                .collect();

            let mut formula_generics = input.generics.clone();
            if !all_field_types.is_empty() && !input.generics.params.is_empty() {
                let predicates = all_field_types.iter().map(|ty| -> syn::WherePredicate {
                    syn::parse_quote_spanned! { ty.span() => #ty: ::alkahest::UnsizedFormula }
                });

                let where_clause = formula_generics.make_where_clause();
                where_clause.predicates.extend(predicates);
            }

            let field_check_names: Vec<Vec<syn::Ident>> = data
                .variants
                .iter()
                .map(|variant| match &variant.fields {
                    syn::Fields::Named(fields) => fields
                        .named
                        .iter()
                        .map(|field| {
                            quote::format_ident!(
                                "__alkahest_formula_variant_{}_field_{}_idx_is",
                                variant.ident,
                                field.ident.as_ref().unwrap(),
                            )
                        })
                        .collect(),
                    _ => Vec::new(),
                })
                .collect();

            let field_check_ids: Vec<Vec<usize>> = data
                .variants
                .iter()
                .map(|variant| match &variant.fields {
                    syn::Fields::Named(fields) => (0..fields.named.len()).collect(),
                    _ => Vec::new(),
                })
                .collect();

            let field_count: Vec<usize> = data
                .variants
                .iter()
                .map(|variant| variant.fields.len())
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

            let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

            let (formula_impl_generics, formula_type_generics, formula_where_clause) =
                formula_generics.split_for_impl();

            Ok(quote::quote! {
                impl #impl_generics #ident #type_generics #where_clause {
                    #(#(
                        #[doc(hidden)]
                        #[inline(always)]
                        pub const fn #field_check_names() -> [(); #field_check_ids] {
                            [(); #field_check_ids]
                        }
                    )*)*

                    #(
                        #[doc(hidden)]
                        #[inline(always)]
                        pub const fn #field_count_checks() -> [(); #field_count] {
                            [(); #field_count]
                        }
                    )*
                }

                impl #formula_impl_generics ::alkahest::Formula for #ident #formula_type_generics #formula_where_clause {}
            })
        }
    }
}
