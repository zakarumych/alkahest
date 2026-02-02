use std::{borrow::Cow, convert::identity};

use proc_easy::EasyMaybe;
use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::{spanned::Spanned, token::Token};

use crate::attrs::SerializeArgs;

pub(crate) fn derive(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(item as syn::DeriveInput);

    match derive_impl(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => proc_macro::TokenStream::from(err.to_compile_error()),
    }
}

fn get_args(attrs: &[syn::Attribute]) -> syn::Result<SerializeArgs> {
    for attr in attrs {
        if attr.path().is_ident("alkahest") {
            let args = attr.parse_args::<SerializeArgs>()?;
            return Ok(args);
        }
    }

    Err(syn::Error::new(
        proc_macro2::Span::call_site(),
        "Missing #[alkahest(...)] attribute that specifies formula and other options",
    ))
}

fn field_member(idx: usize, field: &syn::Field) -> syn::Member {
    match &field.ident {
        Some(ident) => syn::Member::Named(ident.clone()),
        None => syn::Member::Unnamed(syn::Index {
            index: idx as u32,
            span: field.span(),
        }),
    }
}

fn with_element(
    formula: &syn::Path,
    variant: Option<&syn::Ident>,
    idx: usize,
    field: &syn::Field,
) -> TokenStream {
    let mut formula_name = formula.clone();
    formula_name.segments.last_mut().unwrap().arguments = syn::PathArguments::None;

    match variant {
        None => match &field.ident {
            Some(ident) => quote::quote! {
                ::alkahest::private::__Alkahest_with_element(|#formula_name { #ident, .. }: &#formula| #ident)
            },
            None => {
                let underscores = std::iter::repeat(syn::Token![_](Span::call_site())).take(idx);

                quote::quote! {
                    ::alkahest::private::__Alkahest_with_element(|#formula_name ( #(#underscores,)* field, .. ): &#formula| field)
                }
            }
        },
        Some(variant) => match &field.ident {
            Some(ident) => quote::quote! {
                ::alkahest::private::__Alkahest_with_element(|__formula_ref: &#formula| { match *__formula_ref { #formula_name :: #variant { ref #ident, .. } => #ident, _ => unreachable!() } })
            },
            None => {
                let underscores = std::iter::repeat(syn::Token![_](Span::call_site())).take(idx);

                quote::quote! {
                    ::alkahest::private::__Alkahest_with_element(|__formula_ref: &#formula| { match *__formula_ref { #formula_name :: #variant ( #(#underscores,)* ref field, .. ) => field, _ => unreachable!() } })
                }
            }
        },
    }
}

fn bind_self_fields(
    ty: &syn::Ident,
    variant: Option<&syn::Ident>,
    fields: &syn::Fields,
) -> TokenStream {
    let variant = variant.into_iter();
    match fields {
        syn::Fields::Named(fields) => {
            let iter = fields.named.iter().map(|f| f.ident.as_ref().unwrap());
            quote::quote! {
                #ty #(:: #variant)* { #(#iter),* }
            }
        }
        syn::Fields::Unnamed(fields) => {
            let iter = fields
                .unnamed
                .iter()
                .enumerate()
                .map(|(idx, _)| quote::format_ident!("__Alkahest__field_{}", idx));
            quote::quote! {
                #ty #(:: #variant)* ( #(#iter),* )
            }
        }
        syn::Fields::Unit => TokenStream::new(),
    }
}

fn bound_field(idx: usize, field: &syn::Field) -> Cow<'_, syn::Ident> {
    match &field.ident {
        Some(ident) => Cow::Borrowed(ident),
        None => Cow::Owned(quote::format_ident!("__Alkahest__field_{}", idx)),
    }
}

fn derive_impl(mut input: syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let args = get_args(&input.attrs)?;

    let formula = &args.formula;
    let name = &input.ident;
    let variant = match &args.variant {
        EasyMaybe::Nothing => None,
        EasyMaybe::Just(variant) => Some(&variant.ident),
    };

    if let Some(custom_predicates) = args.where_clause
        && !custom_predicates.predicates.is_empty()
    {
        input
            .generics
            .make_where_clause()
            .predicates
            .extend(custom_predicates.predicates);
    }

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    match input.data {
        syn::Data::Struct(data) => {
            let variant_idx_serialize = variant.map(|variant| {
                let idx = quote::format_ident!("__Alkahest_DISCRIMINANT_OF_{}", variant);
                quote::quote! {
                    ::alkahest::private::__Alkahest_serialize_discriminant(<#formula>::#idx, <#formula>::__Alkahest_DISCRIMINANT_COUNT, &mut __serializer)?;
                }
            });

            let add_discriminant_size = if variant.is_some() {
                quote::quote! {
                    total.add_stack(::alkahest::private::__Alkahest_discriminant_size(<#formula>::__Alkahest_DISCRIMINANT_COUNT));
                }
            } else {
                quote::quote! {}
            };

            let bind_self_fields = bind_self_fields(name, None, &data.fields);
            let fields = data.fields.iter().enumerate().map(|(idx, field)| {
                let check_idx = field.ident.as_ref().map(|name| {
                    let ident = match variant {
                        None => quote::format_ident!("__Alkahest_ORDER_OF_{}", name),
                        Some(variant) => {
                            quote::format_ident!("__Alkahest_ORDER_OF_{}_{}", variant, name)
                        }
                    };

                    quote::quote! {
                        const { assert!(<#formula>::#ident == #idx); }
                    }
                });

                let with_element = with_element(formula, variant, idx, field);
                let bound_field = bound_field(idx, field);

                let serialize = quote::quote! {
                    #with_element .serialize(#bound_field, &mut __serializer)?;
                };

                let add_size_hint = quote::quote! {
                    total += #with_element.size_hint::<_, __SIZE_BYTES>(#bound_field)?;
                };

                (serialize, add_size_hint, check_idx)
            });

            let (fields_serialize, fields_add_size_hint, fields_check_idx): (
                Vec<_>,
                Vec<_>,
                Vec<_>,
            ) = fields.collect();

            Ok(quote::quote! {
                impl #impl_generics ::alkahest::private::__Alkahest_Serialize<#formula> for #name #ty_generics #where_clause {
                    #[inline]
                    fn serialize<S>(&self, mut __serializer: S) -> ::alkahest::private::__Alkahest_Result<(), S::Error>
                    where
                        S: ::alkahest::private::__Alkahest_Serializer,
                    {
                        #(#fields_check_idx)*

                        let #bind_self_fields = self;
                        #variant_idx_serialize
                        #(#fields_serialize)*
                        Ok(())
                    }

                    #[inline]
                    fn size_hint<const __SIZE_BYTES: u8>(&self) -> Option<::alkahest::private::__Alkahest_Sizes> {
                        let mut total = ::alkahest::private::__Alkahest_Sizes::ZERO;
                        let #bind_self_fields = self;
                        #add_discriminant_size
                        #(#fields_add_size_hint)*
                        ::alkahest::private::__Alkahest_Some(total)
                    }
                }
            })
        }
        syn::Data::Enum(data) => {
            if variant.is_some() {
                return Err(syn::Error::new(
                    input.ident.span(),
                    "Cannot specify variant when deriving Serialize for enums",
                ));
            }

            let add_discriminant_size = if variant.is_some() {
                quote::quote! {
                    total.add_stack(<#formula>::__Alkahest_DISCRIMINANT_SIZE);
                }
            } else {
                quote::quote! {}
            };

            let variants = data.variants.iter().map(|data_variant| {
                let variant = &data_variant.ident;

                let variant_idx_serialize = {
                    let idx = quote::format_ident!("__Alkahest_ORDER_OF_{}", variant);
                    quote::quote! {
                        ::alkahest::private::__Alkahest_serialize_discriminant(#idx, <#formula>::__Alkahest_DISCRIMINANT_SIZE, &mut __serializer)?;
                    }
                };

                let fields = data_variant.fields.iter().enumerate().map(|(idx, field)| {
                    let check_idx = field.ident.as_ref().map(|name| {
                        let ident = quote::format_ident!("__Alkahest_ORDER_OF_{}_{}", variant, name);

                        quote::quote! {
                            const { assert!(<#formula>::#ident == #idx); }
                        }
                    });

                    let with_element = with_element(formula, Some(variant), idx, field);
                    let bound_field = bound_field(idx, field);

                    let serialize = quote::quote! {
                        #with_element.serialize(#bound_field, &mut __serializer)?;
                    };

                    let add_size_hint = quote::quote! {
                        total += #with_element.size_hint::<_, __SIZE_BYTES>(#bound_field)?;
                    };

                    (serialize, add_size_hint, check_idx)
                });

                let (fields_serialize, fields_add_size_hint, fields_check_idx): (Vec<_>, Vec<_>, Vec<_>) = fields.collect();

                let bind_self_fields = bind_self_fields(name, Some(variant), &data_variant.fields);

                let variant_serialize = quote::quote! {
                    #bind_self_fields => {
                        #(#fields_check_idx)*

                        let #bind_self_fields = self;
                        #variant_idx_serialize
                        #(#fields_serialize)*
                        Ok(())
                    }
                };

                let variant_size_hint = quote::quote! {
                    #bind_self_fields => {
                        let mut total = ::alkahest::private::__Alkahest_Sizes::ZERO;
                        let #bind_self_fields = self;
                        #add_discriminant_size
                        #(#fields_add_size_hint)*
                        ::alkahest::private::__Alkahest_Some(total)
                    }
                };

                (variant_serialize, variant_size_hint)
            });

            let (variants_serialize, variants_size_hint): (Vec<_>, Vec<_>) = variants.unzip();

            Ok(quote::quote! {
                impl #impl_generics ::alkahest::private::__Alkahest_Serialize for #name #ty_generics #where_clause {
                    #[inline]
                    fn serialize<S>(&self, mut __serializer: S) -> ::alkahest::private::__Alkahest_Result<(), S::Error>
                    where
                        S: ::alkahest::private::__Alkahest_Serializer,
                    {
                        match self {
                            #(#variants_serialize)*
                        }
                    }

                    #[inline]
                    fn size_hint<const __SIZE_BYTES: u8>(&self) -> Option<::alkahest::private::__Alkahest_Sizes> {
                        match self {
                            #(#variants_size_hint)*
                        }
                    }
                }
            })
        }
        _ => {
            return Err(syn::Error::new(
                input.ident.span(),
                "Serialize can only be derived for structs and enums",
            ));
        }
    }
}
