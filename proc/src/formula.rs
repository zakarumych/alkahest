use std::collections::HashSet;

use proc_macro2::TokenStream;
use syn::spanned::Spanned;

use crate::{attrs::FormulaArgs, filter_type_param, is_generic_ty};

struct Config {
    formula_generics: syn::Generics,
}

impl Config {
    pub fn from_args(args: FormulaArgs, generics: &syn::Generics, data: &syn::Data) -> Self {
        let formula_generics = match args.generics {
            None => {
                let all_field_types: Vec<_> = match data {
                    syn::Data::Struct(data) => data.fields.iter().map(|field| &field.ty).collect(),
                    syn::Data::Enum(data) => data
                        .variants
                        .iter()
                        .flat_map(|variant| variant.fields.iter().map(|field| &field.ty))
                        .collect(),
                    syn::Data::Union(_) => {
                        panic!("Alkahest does not support unions");
                    }
                };

                let mut all_generic_field_types: HashSet<_> =
                    all_field_types.iter().copied().collect();

                all_generic_field_types
                    .retain(|ty| is_generic_ty(ty, &filter_type_param(generics.params.iter())));

                let mut formula_generics = generics.clone();
                if !all_generic_field_types.is_empty() {
                    let predicates = all_generic_field_types
                    .iter()
                    .map(|ty| -> syn::WherePredicate {
                        syn::parse_quote_spanned! { ty.span() => #ty: ::alkahest::private::FormulaType }
                    });
                    let where_clause = formula_generics.make_where_clause();
                    where_clause.predicates.extend(predicates);
                };

                formula_generics
            }
            Some(args_generics) => {
                let mut formula_generics = generics.clone();
                formula_generics.params.extend(args_generics.params);
                if let Some(where_clause) = args_generics.where_clause {
                    formula_generics
                        .make_where_clause()
                        .predicates
                        .extend(where_clause.predicates);
                }
                formula_generics
            }
        };

        Config { formula_generics }
    }
}

#[allow(clippy::too_many_lines)]
pub fn derive(args: FormulaArgs, input: &syn::DeriveInput) -> syn::Result<TokenStream> {
    let ident = &input.ident;

    let config = Config::from_args(args, &input.generics, &input.data);

    match &input.data {
        syn::Data::Union(data) => Err(syn::Error::new_spanned(
            data.union_token,
            "Formula cannot be derived for unions",
        )),
        syn::Data::Struct(data) => {
            let all_field_types: Vec<_> = data.fields.iter().map(|field| &field.ty).collect();
            let last_field_type = all_field_types.last().copied().into_iter();

            let field_names_order = match &data.fields {
                syn::Fields::Named(fields) => fields
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

            let field_ids: Vec<_> = (0..data.fields.len()).collect();

            let (formula_impl_generics, formula_type_generics, formula_where_clause) =
                config.formula_generics.split_for_impl();

            let touch_fields = match &data.fields {
                syn::Fields::Unit => quote::quote! {},
                syn::Fields::Unnamed(fields) => {
                    let fields = (0..fields.unnamed.len()).map(|idx| {
                        let ident = quote::format_ident!("__{}", idx);
                        quote::quote! { #ident }
                    });
                    quote::quote! { ( #(#fields),* ) }
                }
                syn::Fields::Named(fields) => {
                    let fields = fields.named.iter().map(|field| {
                        let ident = &field.ident;
                        quote::quote! { #ident }
                    });
                    quote::quote_spanned! { data.fields.span() => { #(#fields),* } }
                }
            };

            let tokens = quote::quote! {
                impl #formula_impl_generics #ident #formula_type_generics #formula_where_clause {
                    #(
                        #[doc(hidden)]
                        #[allow(non_upper_case_globals)]
                        pub const #field_names_order: ::alkahest::private::usize = #field_ids;
                    )*

                    // #(#with_fields)*

                    #[doc(hidden)]
                    #[allow(dead_code, unused_variables)]
                    fn __alkahest_touch(&self) {
                        let Self #touch_fields = self;
                    }
                }

                impl #formula_impl_generics ::alkahest::private::FormulaType for #ident #formula_type_generics #formula_where_clause {
                    const MAX_STACK_SIZE: ::alkahest::private::Option<::alkahest::private::usize> = {
                        #[allow(unused_mut)]
                        let mut max_size = Some(0);
                        #(
                            max_size = ::alkahest::private::sum_size(max_size, <#all_field_types as ::alkahest::private::FormulaType>::MAX_STACK_SIZE);
                        )*;
                        // #expand_size
                        max_size
                    };

                    const EXACT_SIZE: ::alkahest::private::bool = {true #(; <#last_field_type as ::alkahest::private::FormulaType>::EXACT_SIZE)*};

                    const HEAPLESS: ::alkahest::private::bool = true #(&& <#all_field_types as ::alkahest::private::FormulaType>::HEAPLESS)*;
                }

                impl #formula_impl_generics ::alkahest::private::BareFormulaType for #ident #formula_type_generics #formula_where_clause {}
            };

            Ok(tokens)
        }
        syn::Data::Enum(data) => {
            let all_field_types: Vec<Vec<&syn::Type>> = data
                .variants
                .iter()
                .map(|variant| variant.fields.iter().map(|field| &field.ty).collect())
                .collect();

            let last_field_types: Vec<Vec<_>> = all_field_types
                .iter()
                .map(|variants| variants.last().copied().into_iter().collect())
                .collect();

            let field_names_order: Vec<Vec<syn::Ident>> = data
                .variants
                .iter()
                .map(|variant| match &variant.fields {
                    syn::Fields::Named(fields) => fields
                        .named
                        .iter()
                        .map(|field| {
                            quote::format_ident!(
                                "__ALKAHEST_FORMULA_VARIANT_{}_FIELD_{}_IDX",
                                variant.ident,
                                field.ident.as_ref().unwrap(),
                            )
                        })
                        .collect(),
                    _ => Vec::new(),
                })
                .collect();

            let field_ids: Vec<Vec<usize>> = data
                .variants
                .iter()
                .map(|v| (0..v.fields.len()).collect())
                .collect();

            let variant_name_ids: Vec<syn::Ident> = data
                .variants
                .iter()
                .map(|v| quote::format_ident!("__ALKAHEST_FORMULA_VARIANT_{}_IDX", v.ident))
                .collect();

            #[allow(clippy::cast_possible_truncation)]
            let variant_ids: Vec<_> = (0..data.variants.len() as u32).collect();

            let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

            let (formula_impl_generics, formula_type_generics, formula_where_clause) =
                config.formula_generics.split_for_impl();

            // let expand_size = if non_exhaustive {
            //     quote::quote! {
            //         max_size = ::alkahest::private::Option::None;
            //     }
            // } else {
            //     quote::quote! {}
            // };

            let touch_variants = data
                .variants
                .iter()
                .map(|v| {
                    let variant_ident = &v.ident;
                    match &v.fields {
                        syn::Fields::Unit => quote::quote! {
                            Self :: #variant_ident => {}
                        },
                        syn::Fields::Unnamed(fields) => {
                            let fields = (0..fields.unnamed.len()).map(|idx| {
                                let ident = quote::format_ident!("__{}", idx);
                                quote::quote! { #ident }
                            });
                            quote::quote! { Self :: #variant_ident ( #(#fields),* ) => {} }
                        }
                        syn::Fields::Named(fields) => {
                            let fields = fields.named.iter().map(|field| {
                                let ident = &field.ident;
                                quote::quote! { #ident }
                            });
                            quote::quote! { Self :: #variant_ident { #(#fields),* } => {} }
                        }
                    }
                })
                .collect::<Vec<_>>();

            let construct_variants = data
                .variants
                .iter()
                .map(|v| {
                    let variant_ident = &v.ident;
                    match &v.fields {
                        syn::Fields::Unit => quote::quote! {
                            let _ = Self :: #variant_ident;
                        },
                        syn::Fields::Unnamed(fields) => {
                            let fields =
                                (0..fields.unnamed.len()).map(|_| quote::quote! { fake() });
                            quote::quote! { let _ = Self :: #variant_ident ( #(#fields),* ); }
                        }
                        syn::Fields::Named(fields) => {
                            let fields = fields.named.iter().map(|field| {
                                let ident = &field.ident;
                                quote::quote! { #ident: fake() }
                            });
                            quote::quote! { let _ = Self :: #variant_ident { #(#fields),* }; }
                        }
                    }
                })
                .collect::<Vec<_>>();

            Ok(quote::quote! {
                impl #impl_generics #ident #type_generics #where_clause {
                    #(#(
                        #[doc(hidden)]
                        #[allow(non_upper_case_globals)]
                        pub const #field_names_order: ::alkahest::private::usize = #field_ids;
                    )*)*

                    #(
                        #[doc(hidden)]
                        #[allow(non_upper_case_globals)]
                        pub const #variant_name_ids: u32 = #variant_ids;
                    )*

                    #[doc(hidden)]
                    #[allow(dead_code, unused_variables)]
                    fn __alkahest_touch(&self) {
                        match self {
                            #( #touch_variants )*
                        }
                    }

                    #[doc(hidden)]
                    #[allow(dead_code, unused_variables)]
                    fn __alkahest_construct() {
                        fn fake<T>() -> T { loop {} }
                        #(#construct_variants)*
                    }
                }

                impl #formula_impl_generics ::alkahest::private::FormulaType for #ident #formula_type_generics #formula_where_clause {
                    const MAX_STACK_SIZE: ::alkahest::private::Option<::alkahest::private::usize> = {
                        #[allow(unused_mut)]
                        let mut max_size = Some(0);

                        #(
                            let var_size = {
                                #[allow(unused_mut)]
                                let mut max_size = Some(0);
                                #(
                                    max_size = ::alkahest::private::sum_size(max_size, <#all_field_types as ::alkahest::private::FormulaType>::MAX_STACK_SIZE);
                                )*;
                                max_size
                            };
                            max_size = ::alkahest::private::max_size(max_size, var_size);
                        )*

                        // #expand_size
                        ::alkahest::private::sum_size(::alkahest::private::VARIANT_SIZE_OPT, max_size)
                    };

                    #[allow(unused_assignments)]
                    const EXACT_SIZE: ::alkahest::private::bool = true && {
                        let mut exact = true;
                        let mut common_size = None;
                        #(
                            #(exact &= <#last_field_types as ::alkahest::private::FormulaType>::EXACT_SIZE;)*

                            let var_size = {
                                #[allow(unused_mut)]
                                let mut max_size = Some(0);
                                #(
                                    max_size = ::alkahest::private::sum_size(max_size, <#all_field_types as ::alkahest::private::FormulaType>::MAX_STACK_SIZE);
                                )*;
                                max_size
                            };
                            exact &= match (common_size, var_size) {
                                (_, None) => false,
                                (None, _) => true,
                                (Some(common_size), Some(var_size)) => common_size == var_size,
                            };
                            common_size = var_size;
                        )*
                        exact
                    };

                    const HEAPLESS: ::alkahest::private::bool = true #(#(&& <#all_field_types as ::alkahest::private::FormulaType>::HEAPLESS)*)*;
                }

                impl #formula_impl_generics ::alkahest::private::BareFormulaType for #ident #formula_type_generics #formula_where_clause {}
            })
        }
    }
}
