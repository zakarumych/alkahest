use std::borrow::Cow;

use proc_macro2::{Span, TokenStream};

use crate::args::{DeserializeArgs, DeserializeDeriveArgs};

pub(crate) fn derive(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(item as syn::DeriveInput);

    let args = match get_args(&input.attrs) {
        Ok(args) => DeserializeArgs::from_derive(args),
        Err(err) => {
            return proc_macro::TokenStream::from(err.to_compile_error());
        }
    };

    match derive_impl(input, args) {
        Ok(tokens) => tokens.into(),
        Err(err) => proc_macro::TokenStream::from(err.to_compile_error()),
    }
}

fn get_args(attrs: &[syn::Attribute]) -> syn::Result<DeserializeDeriveArgs> {
    for attr in attrs {
        if attr.path().is_ident("alkahest") {
            let args = attr.parse_args::<DeserializeDeriveArgs>()?;
            return Ok(args);
        }
    }

    Ok(DeserializeDeriveArgs::default())
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
    args: DeserializeArgs,
) -> syn::Result<proc_macro2::TokenStream> {
    let formula = &args.formula;
    let name = &input.ident;

    let deserialize_generics = args.merge_generics(&input.generics);
    let lifetime_generics = args.with_de_lifetime(&deserialize_generics);
    let deserializer_lifetime = args.deserializer_lifetime();

    let (_impl_generics, ty_generics, _where_clause) = input.generics.split_for_impl();
    let (_impl_generics, _ty_generics, where_clause) = deserialize_generics.split_for_impl();
    let (impl_generics, _ty_generics, _where_clause) = lifetime_generics.split_for_impl();

    match input.data {
        syn::Data::Struct(data) => {
            let bind_self_fields = bind_self_fields(name, None, &data.fields);
            let fields = data.fields.iter().enumerate().map(|(idx, field)| {
                let check_idx = field.ident.as_ref().map(|name| {
                    let ident = quote::format_ident!("__ALKAHEST_ORDER_OF_{}", name);

                    quote::quote! {
                        const { assert!(<#formula>::#ident == #idx); }
                    }
                });

                let with_element = with_element(formula, None, idx, field);
                let bound_field = bound_field(idx, field);

                let deserialize = quote::quote! {
                    let #bound_field = #with_element .deserialize(&mut __deserializer)?;
                };

                let deserialize_in_place = quote::quote! {
                    #with_element .deserialize_in_place(#bound_field, &mut __deserializer)?;
                };

                (deserialize, deserialize_in_place, check_idx)
            });

            let (fields_deserialize, fields_deserialize_in_place, fields_check_idx): (
                Vec<_>,
                Vec<_>,
                Vec<_>,
            ) = fields.collect();

            Ok(quote::quote! {
                impl #impl_generics ::alkahest::Deserialize<#deserializer_lifetime, #formula> for #name #ty_generics #where_clause {
                    #[inline]
                    fn deserialize<__Deserializer>(mut __deserializer: __Deserializer) -> ::alkahest::private::Result<Self, ::alkahest::DeserializeError>
                    where
                        __Deserializer: ::alkahest::Deserializer<#deserializer_lifetime>,
                    {
                        #(#fields_check_idx)*
                        #(#fields_deserialize)*
                        ::alkahest::private::Ok(#bind_self_fields)
                    }

                    #[inline]
                    fn deserialize_in_place<__Deserializer>(&mut self, mut __deserializer: __Deserializer) -> ::alkahest::private::Result<(), ::alkahest::DeserializeError>
                    where
                        __Deserializer: ::alkahest::Deserializer<#deserializer_lifetime>,
                    {
                        #(#fields_check_idx)*
                        let #bind_self_fields = self;
                        #(#fields_deserialize_in_place)*
                        ::alkahest::private::Ok(())
                    }
                }
            })
        }
        syn::Data::Enum(data) => {
            let deserialize_discriminant = quote::quote! {
                let __discriminant = ::alkahest::private::deserialize_discriminant(<#formula>::__ALKAHEST_DISCRIMINANT_COUNT, &mut __deserializer)?;
            };

            let variants = data.variants.iter().map(|data_variant| {
                let variant = &data_variant.ident;

                let variant_idx_check = {
                    let idx = quote::format_ident!("__ALKAHEST_DISCRIMINANT_OF_{}", variant);

                    quote::quote! {
                        __discriminant == <#formula>::#idx
                    }
                };

                let fields = data_variant.fields.iter().enumerate().map(|(idx, field)| {
                    let check_idx = field.ident.as_ref().map(|name| {
                        let ident =
                            quote::format_ident!("__ALKAHEST_ORDER_OF_{}_{}", variant, name);

                        quote::quote! {
                            const { assert!(<#formula>::#ident == #idx); }
                        }
                    });

                    let with_element = with_element(formula, Some(variant), idx, field);
                    let bound_field = bound_field(idx, field);

                    let deserialize = quote::quote! {
                        let #bound_field = #with_element.deserialize(&mut __deserializer)?;
                    };

                    let deserialize_in_place = quote::quote! {
                        #with_element.deserialize_in_place(#bound_field, &mut __deserializer)?;
                    };

                    (deserialize, deserialize_in_place, check_idx)
                });

                let (fields_deserialize, fields_deserialize_in_place, fields_check_idx): (
                    Vec<_>,
                    Vec<_>,
                    Vec<_>,
                ) = fields.collect();

                let bind_self_fields = bind_self_fields(name, Some(variant), &data_variant.fields);

                let variant_deserialize = quote::quote! {
                    if #variant_idx_check {
                        #(#fields_check_idx)*
                        #(#fields_deserialize)*
                        return ::alkahest::private::Ok(#bind_self_fields);
                    }
                };

                let variant_deserialize_in_place = quote::quote! {
                    #bind_self_fields if #variant_idx_check => {
                        #(#fields_check_idx)*
                        #(#fields_deserialize_in_place)*
                        return ::alkahest::private::Ok(());
                    }
                };

                (variant_deserialize, variant_deserialize_in_place)
            });

            let (variants_deserialize, variants_deserialize_in_place): (Vec<_>, Vec<_>) =
                variants.unzip();

            Ok(quote::quote! {
                impl #impl_generics ::alkahest::private::DeserializeEnumVariant<#deserializer_lifetime, #formula> for #name #ty_generics #where_clause {
                    #[inline]
                    fn deserialize_enum_variant<__Deserializer>(__discriminant: usize, mut __deserializer: __Deserializer) -> ::alkahest::private::Result<Self, ::alkahest::DeserializeError>
                    where
                        __Deserializer: ::alkahest::Deserializer<#deserializer_lifetime>,
                    {
                        #(#variants_deserialize)*
                        ::alkahest::private::Err(::alkahest::DeserializeError::WrongVariant(__discriminant))
                    }
                }

                impl #impl_generics ::alkahest::Deserialize<#deserializer_lifetime, #formula> for #name #ty_generics #where_clause {
                    #[inline]
                    fn deserialize<__Deserializer>(mut __deserializer: __Deserializer) -> ::alkahest::private::Result<Self, ::alkahest::DeserializeError>
                    where
                        __Deserializer: ::alkahest::Deserializer<#deserializer_lifetime>,
                    {
                        #deserialize_discriminant
                        <Self as ::alkahest::private::DeserializeEnumVariant<#deserializer_lifetime, #formula>>::deserialize_enum_variant(__discriminant, __deserializer)
                    }

                    #[inline]
                    fn deserialize_in_place<__Deserializer>(&mut self, mut __deserializer: __Deserializer) -> ::alkahest::private::Result<(), ::alkahest::DeserializeError>
                    where
                        __Deserializer: ::alkahest::Deserializer<#deserializer_lifetime>,
                    {
                        #deserialize_discriminant
                        match self {
                            #(#variants_deserialize_in_place)*
                            _ => {
                                // Different variant, cannot deserialize in place
                                *self = <Self as ::alkahest::private::DeserializeEnumVariant<#deserializer_lifetime, #formula>>::deserialize_enum_variant(__discriminant, __deserializer)?;
                                ::alkahest::private::Ok(())
                            }
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
