use std::borrow::Cow;

use proc_easy::EasyMaybe;
use proc_macro2::{Span, TokenStream};

use crate::args::{SerializeArgs, SerializeDeriveArgs};

pub(crate) fn derive(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(item as syn::DeriveInput);

    let args = match get_args(&input.attrs) {
        Ok(args) => SerializeArgs::from_derive(args),
        Err(err) => {
            return proc_macro::TokenStream::from(err.to_compile_error());
        }
    };

    match derive_impl(input, args) {
        Ok(tokens) => tokens.into(),
        Err(err) => proc_macro::TokenStream::from(err.to_compile_error()),
    }
}

fn get_args(attrs: &[syn::Attribute]) -> syn::Result<SerializeDeriveArgs> {
    for attr in attrs {
        if attr.path().is_ident("alkahest") {
            let args = attr.parse_args::<SerializeDeriveArgs>()?;
            return Ok(args);
        }
    }

    Ok(SerializeDeriveArgs::default())
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
                ::alkahest::private::with_element(|#formula_name { #ident, .. }: &#formula| #ident)
            },
            None => {
                let underscores = std::iter::repeat(syn::Token![_](Span::call_site())).take(idx);

                quote::quote! {
                    ::alkahest::private::with_element(|#formula_name ( #(#underscores,)* field, .. ): &#formula| field)
                }
            }
        },
        Some(variant) => match &field.ident {
            Some(ident) => quote::quote! {
                ::alkahest::private::with_element(|__formula_ref: &#formula| { match *__formula_ref { #formula_name :: #variant { ref #ident, .. } => #ident, _ => unreachable!() } })
            },
            None => {
                let underscores = std::iter::repeat(syn::Token![_](Span::call_site())).take(idx);

                quote::quote! {
                    ::alkahest::private::with_element(|__formula_ref: &#formula| { match *__formula_ref { #formula_name :: #variant ( #(#underscores,)* ref field, .. ) => field, _ => unreachable!() } })
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
                .map(|(idx, _)| quote::format_ident!("__alkahest_field_{}", idx));
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
        None => Cow::Owned(quote::format_ident!("__alkahest_field_{}", idx)),
    }
}

pub fn derive_impl(
    input: syn::DeriveInput,
    args: SerializeArgs,
) -> syn::Result<proc_macro2::TokenStream> {
    let formula = &args.formula;
    let name = &input.ident;
    let variant = match &args.variant {
        EasyMaybe::Nothing => None,
        EasyMaybe::Just(variant) => Some(&variant.ident),
    };

    let serialize_generics = args.merge_generics(&input.generics);

    let (_impl_generics, ty_generics, _where_clause) = input.generics.split_for_impl();
    let (impl_generics, _ty_generics, where_clause) = serialize_generics.split_for_impl();

    match input.data {
        syn::Data::Struct(data) => {
            let variant_idx_serialize = variant.map(|variant| {
                let idx = quote::format_ident!("__ALKAHEST_DISCRIMINANT_OF_{}", variant);
                quote::quote! {
                    ::alkahest::private::serialize_discriminant(<#formula>::#idx, <#formula>::__ALKAHEST_DISCRIMINANT_COUNT, &mut __serializer)?;
                }
            });

            let add_discriminant_size = if variant.is_some() {
                quote::quote! {
                    __total_size.add_stack(::alkahest::private::discriminant_size(<#formula>::__ALKAHEST_DISCRIMINANT_COUNT));
                }
            } else {
                quote::quote! {}
            };

            let bind_self_fields = bind_self_fields(name, None, &data.fields);
            let fields = data.fields.iter().enumerate().map(|(idx, field)| {
                let check_idx = field.ident.as_ref().map(|name| {
                    let ident = match variant {
                        None => quote::format_ident!("__ALKAHEST_ORDER_OF_{}", name),
                        Some(variant) => {
                            quote::format_ident!("__ALKAHEST_ORDER_OF_{}_{}", variant, name)
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
                    __total_size += #with_element.size_hint::<_, __SIZE_BYTES>(#bound_field)?;
                };

                (serialize, add_size_hint, check_idx)
            });

            let (fields_serialize, fields_add_size_hint, fields_check_idx): (
                Vec<_>,
                Vec<_>,
                Vec<_>,
            ) = fields.collect();

            Ok(quote::quote! {
                impl #impl_generics ::alkahest::Serialize<#formula> for #name #ty_generics #where_clause {
                    #[inline]
                    fn serialize<__Serializer>(&self, mut __serializer: __Serializer) -> ::alkahest::private::Result<(), __Serializer::Error>
                    where
                        __Serializer: ::alkahest::Serializer,
                    {
                        #(#fields_check_idx)*

                        let #bind_self_fields = self;
                        #variant_idx_serialize
                        #(#fields_serialize)*
                        Ok(())
                    }

                    #[inline]
                    fn size_hint<const __SIZE_BYTES: u8>(&self) -> Option<::alkahest::Sizes> {
                        let mut __total_size = ::alkahest::Sizes::ZERO;
                        let #bind_self_fields = self;
                        #add_discriminant_size
                        #(#fields_add_size_hint)*
                        ::alkahest::private::Some(__total_size)
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

            let add_discriminant_size = quote::quote! {
                __total_size.add_stack(const { ::alkahest::private::discriminant_size(<#formula>::__ALKAHEST_DISCRIMINANT_COUNT) });
            };

            let variants = data.variants.iter().map(|data_variant| {
                let variant = &data_variant.ident;

                let variant_idx_serialize = {
                    let idx = quote::format_ident!("__ALKAHEST_DISCRIMINANT_OF_{}", variant);
                    quote::quote! {
                        ::alkahest::private::serialize_discriminant(<#formula>::#idx, <#formula>::__ALKAHEST_DISCRIMINANT_COUNT, &mut __serializer)?;
                    }
                };

                let fields = data_variant.fields.iter().enumerate().map(|(idx, field)| {
                    let check_idx = field.ident.as_ref().map(|name| {
                        let ident = quote::format_ident!("__ALKAHEST_ORDER_OF_{}_{}", variant, name);

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
                        __total_size += #with_element.size_hint::<_, __SIZE_BYTES>(#bound_field)?;
                    };

                    (serialize, add_size_hint, check_idx)
                });

                let (fields_serialize, fields_add_size_hint, fields_check_idx): (Vec<_>, Vec<_>, Vec<_>) = fields.collect();

                let bind_self_fields = bind_self_fields(name, Some(variant), &data_variant.fields);

                let variant_serialize = quote::quote! {
                    #bind_self_fields => {
                        #(#fields_check_idx)*
                        #variant_idx_serialize
                        #(#fields_serialize)*
                        Ok(())
                    }
                };

                let variant_size_hint = quote::quote! {
                    #bind_self_fields => {
                        #(#fields_add_size_hint)*
                    }
                };

                (variant_serialize, variant_size_hint)
            });

            let (variants_serialize, variants_size_hint): (Vec<_>, Vec<_>) = variants.unzip();

            Ok(quote::quote! {
                impl #impl_generics ::alkahest::Serialize<#formula> for #name #ty_generics #where_clause {
                    #[inline]
                    fn serialize<__Serializer>(&self, mut __serializer: __Serializer) -> ::alkahest::private::Result<(), __Serializer::Error>
                    where
                        __Serializer: ::alkahest::Serializer,
                    {
                        match self {
                            #(#variants_serialize)*
                        }
                    }

                    #[inline]
                    fn size_hint<const __SIZE_BYTES: u8>(&self) -> Option<::alkahest::Sizes> {
                        let mut __total_size = ::alkahest::Sizes::ZERO;
                        #add_discriminant_size

                        match self {
                            #(#variants_size_hint)*
                        }

                        ::alkahest::private::Some(__total_size)
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
