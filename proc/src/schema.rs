use {proc_macro2::TokenStream, syn::spanned::Spanned, std::convert::TryFrom};

pub fn derive_schema(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    let vis = &input.vis;

    let ident = &input.ident;
    let packed_ident = quote::format_ident!("{}Packed", input.ident);
    let unpacked_ident = quote::format_ident!("{}Unpacked", input.ident);

    for param in &input.generics.params {
        if let syn::GenericParam::Lifetime(lifetime) = param {
            return Err(syn::Error::new_spanned(
                lifetime,
                "Schema derive macro does not support structures with lifetime parameters",
            ));
        }
    }

    let generics = &input.generics;
    let mut unpacked_generics = input.generics.clone();

    
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    let lt_token = *unpacked_generics
        .lt_token
        .get_or_insert_with(Default::default);

    let gt_token = *unpacked_generics
        .gt_token
        .get_or_insert_with(Default::default);

    unpacked_generics
        .params
        .push(syn::GenericParam::Lifetime(syn::LifetimeDef::new(
            syn::Lifetime::new("'alkahest", proc_macro2::Span::call_site()),
        )));

    let result = match input.data {
        syn::Data::Enum(data) => {
            let packed_variants_ident = quote::format_ident!("{}PackedVariants", input.ident);

            let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
            let (unpacked_impl_generics, unpacked_type_generics, _) =
                unpacked_generics.split_for_impl();

            let align_masks = data.variants.iter().flat_map(|variant| {
                variant.fields.iter().map(|field| {
                    let ty = &field.ty;
                    quote::quote_spanned!(field.span() => (<#ty as ::alkahest::Schema>::align() - 1))
                })
            });

            let packed_variants = data.variants.iter().map(|variant| {
                let packed_variant_ident = quote::format_ident!("{}{}Packed", ident, variant.ident);

                let packed_variant_generics = if variant.fields.is_empty() {
                    syn::Generics::default()
                } else {
                    let bounds: syn::punctuated::Punctuated<_, _> = std::array::IntoIter::new([
                        syn::TypeParamBound::Trait(syn::TraitBound {
                            paren_token: None,
                            modifier: syn::TraitBoundModifier::None,
                            lifetimes: None,
                            path: syn::parse2(quote::quote!(::core::clone::Clone)).unwrap(),
                        }),
                        syn::TypeParamBound::Trait(syn::TraitBound {
                            paren_token: None,
                            modifier: syn::TraitBoundModifier::None,
                            lifetimes: None,
                            path: syn::parse2(quote::quote!(::core::marker::Copy)).unwrap(),
                        }),
                        syn::TypeParamBound::Trait(syn::TraitBound {
                            paren_token: None,
                            modifier: syn::TraitBoundModifier::None,
                            lifetimes: None,
                            path: syn::parse2(quote::quote!(::alkahest::Zeroable)).unwrap(),
                        }),
                        syn::TypeParamBound::Trait(syn::TraitBound {
                            paren_token: None,
                            modifier: syn::TraitBoundModifier::None,
                            lifetimes: None,
                            path: syn::parse2(quote::quote!(::alkahest::Pod)).unwrap(),
                        }),
                    ]).collect();

                    syn::Generics {
                        lt_token: Some(lt_token),
                        params: (0..variant.fields.len())
                            .map(|idx| {
                                syn::GenericParam::Type(syn::TypeParam {
                                    ident: quote::format_ident!("PackedField{}", idx),
                                    attrs: Vec::new(),
                                    colon_token: None,
                                    bounds: bounds.clone(),
                                    eq_token: None,
                                    default: None,
                                })
                            })
                            .collect(),
                        gt_token: Some(gt_token),
                        where_clause: None,
                    }
                };

                let (packed_variant_impl_generics, packed_variant_type_generics, packed_variant_where_clause) = packed_variant_generics.split_for_impl();

                match &variant.fields {
                    syn::Fields::Unit => quote::quote_spanned!(variant.span() => 
                        #vis struct #packed_variant_ident;

                        impl ::core::clone::Clone for #packed_variant_ident {
                            #[inline]
                            fn clone(&self) -> Self { *self }
                        }

                        impl ::core::marker::Copy for #packed_variant_ident {}

                        unsafe impl ::alkahest::Zeroable for #packed_variant_ident {}
                        unsafe impl ::alkahest::Pod for #packed_variant_ident {}
                    ),
                    syn::Fields::Unnamed(fields) => {
                        let packed_fields = fields.unnamed.iter().enumerate().map(|(idx, field)| {
                            let ty = quote::format_ident!("PackedField{}", idx);
                            quote::quote_spanned!(field.span() => pub #ty )
                        });

                        quote::quote_spanned!(variant.span() => 
                            #[repr(C, packed)] #vis struct #packed_variant_ident #packed_variant_generics ( #(#packed_fields,)* );

                            impl #packed_variant_impl_generics ::core::clone::Clone for #packed_variant_ident #packed_variant_type_generics #packed_variant_where_clause {
                                #[inline]
                                fn clone(&self) -> Self { *self }
                            }

                            impl #packed_variant_impl_generics ::core::marker::Copy for #packed_variant_ident #packed_variant_type_generics #packed_variant_where_clause {}

                            unsafe impl #packed_variant_impl_generics ::alkahest::Zeroable for #packed_variant_ident #packed_variant_type_generics #packed_variant_where_clause {}
                            unsafe impl #packed_variant_impl_generics ::alkahest::Pod for #packed_variant_ident #packed_variant_type_generics #packed_variant_where_clause {}
                        )
                    }
                    syn::Fields::Named(fields) => {
                        let packed_fields = fields.named.iter().enumerate().map(|(idx, field)| {
                            let ty = quote::format_ident!("PackedField{}", idx);
                            let ident = field.ident.as_ref().unwrap();
                            quote::quote_spanned!(field.span() => pub #ident: #ty )
                        });

                        quote::quote_spanned!(variant.span() =>
                            #[repr(C, packed)] #vis struct #packed_variant_ident #packed_variant_generics { #(#packed_fields,)* }

                            impl #packed_variant_impl_generics ::core::clone::Clone for #packed_variant_ident #packed_variant_type_generics #packed_variant_where_clause {
                                #[inline]
                                fn clone(&self) -> Self { *self }
                            }

                            impl #packed_variant_impl_generics ::core::marker::Copy for #packed_variant_ident #packed_variant_type_generics #packed_variant_where_clause {}

                            unsafe impl #packed_variant_impl_generics ::alkahest::Zeroable for #packed_variant_ident #packed_variant_type_generics #packed_variant_where_clause {}
                            unsafe impl #packed_variant_impl_generics ::alkahest::Pod for #packed_variant_ident #packed_variant_type_generics #packed_variant_where_clause {}
                        )
                    }
                }
            });

            let packed_variant_idents = data
                .variants
                .iter()
                .map(|variant| &variant.ident);

            let packed_variant_concrete_types = data
                .variants
                .iter()
                .map(|variant| {
                    let ident = quote::format_ident!("{}{}Packed", ident, variant.ident);

                    syn::Type::Path(syn::TypePath {
                        qself: None,
                        path: syn::Path {
                            leading_colon: None,
                            segments: std::iter::once(syn::PathSegment {
                                ident,
                                arguments: if variant.fields.is_empty() { syn::PathArguments::None } else { syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                                    colon2_token: None,
                                    lt_token,
                                    args: variant.fields.iter().map(|field| syn::GenericArgument::Type({
                                        let ty = &field.ty;
                                        syn::parse2(quote::quote_spanned!(ty.span() => <#ty as ::alkahest::Schema>::Packed)).unwrap()
                                    })).collect(),
                                    gt_token,
                                }) },
                            }).collect(),
                        }
                    })
                });

            let unpacked_variants = data.variants.iter().map(|variant| {
                let variant_ident= &variant.ident;

                match &variant.fields {
                    syn::Fields::Unit => quote::quote_spanned!(variant.span() => #variant_ident),
                    syn::Fields::Unnamed(fields) => {
                        let unpacked_fields = fields.unnamed.iter().map(|field| {
                            let ty = &field.ty;
                            quote::quote_spanned!(field.span() => <#ty as ::alkahest::SchemaUnpack<'alkahest>>::Unpacked )
                        });

                        quote::quote_spanned!(variant.span() => #variant_ident ( #(#unpacked_fields,)* ))
                    }
                    syn::Fields::Named(fields) => {
                        let unpacked_fields = fields.named.iter().map(|field| {
                            let ty = &field.ty;
                            let ident = field.ident.as_ref().unwrap();
                            quote::quote_spanned!(field.span() => #ident: <#ty as ::alkahest::SchemaUnpack<'alkahest>>::Unpacked )
                        });

                        quote::quote_spanned!(variant.span() => #variant_ident { #(#unpacked_fields,)* })
                    }
                }
            });

            let unpack_variants = data.variants.iter().enumerate().map(|(idx, variant)| {
                let variant_ident=  &variant.ident;

                match &variant.fields {
                    syn::Fields::Unit => quote::quote_spanned!(variant.span() => #idx => #unpacked_ident::#variant_ident),
                    syn::Fields::Unnamed(fields) => {
                        let unpack_fields = fields.unnamed.iter().enumerate().map(|(idx, field)| {
                            let ty = &field.ty;
                            let member = syn::Member::Unnamed(syn::Index { index: idx as u32, span: field.span() });
                            quote::quote_spanned!( field.span() => <#ty as ::alkahest::Schema>::unpack(variant.#member, bytes))
                        });
                        quote::quote_spanned!(variant.span() => #idx => {
                            let variant = unsafe { &packed.variants.#variant_ident };
                            #unpacked_ident::#variant_ident ( #( #unpack_fields, )* )
                        })
                    }
                    syn::Fields::Named(fields) => {
                        let unpack_fields = fields.named.iter().map(|field| {
                            let ty = &field.ty;
                            let ident = field.ident.as_ref().unwrap();
                            quote::quote_spanned!( field.span() => #ident: <#ty as ::alkahest::Schema>::unpack(variant.#ident, bytes))
                        });
                        quote::quote_spanned!(variant.span() => #idx => {
                            let variant = unsafe { &packed.variants.#variant_ident };
                            #unpacked_ident::#variant_ident { #( #unpack_fields, )* }
                        })
                    }
                }
            });

            let pack_variants = data.variants.iter().enumerate().map(|(idx, variant)| {
                let idx = u32::try_from(idx).expect("Too many variants");

                let pack_ident = quote::format_ident!("{}{}Pack", ident, variant.ident);
                let variant_ident = &variant.ident;

                let pack_fields = variant.fields.iter().enumerate().map(|(idx, field)| {
                    let mut ty = quote::format_ident!("PackField{}", idx);
                    ty.set_span(field.ty.span());

                    match &field.ident {
                        None => quote::quote_spanned!( field.span() => pub #ty ),
                        Some(ident) => {
                            quote::quote_spanned!( field.span() => pub #ident: #ty )
                        }
                    }
                });

                let pack_type_generics = if variant.fields.is_empty() {
                    syn::Generics::default()
                } else {
                    syn::Generics {
                        lt_token: Some(lt_token),
                        params: (0..variant.fields.len())
                            .map(|idx| {
                                syn::GenericParam::Type(syn::TypeParam {
                                    ident: quote::format_ident!("PackField{}", idx),
                                    attrs: Vec::new(),
                                    colon_token: None,
                                    bounds: Default::default(),
                                    eq_token: None,
                                    default: None,
                                })
                            })
                            .collect(),
                        gt_token: Some(gt_token),
                        where_clause: None,
                    }
                };

                let mut pack_generics = generics.clone();

                pack_generics
                    .params
                    .extend((0..variant.fields.len()).map(|idx| {
                        syn::GenericParam::Type(syn::TypeParam {
                            ident: quote::format_ident!("PackField{}", idx),
                            attrs: Vec::new(),
                            colon_token: None,
                            bounds: Default::default(),
                            eq_token: None,
                            default: None,
                        })
                    }));

                if !variant.fields.is_empty() {
                    let pack_where_clause =
                        pack_generics
                            .where_clause
                            .get_or_insert_with(|| syn::WhereClause {
                                where_token: Default::default(),
                                predicates: Default::default(),
                            });

                    pack_where_clause
                        .predicates
                        .extend(variant.fields.iter().enumerate().map(|(idx, field)| {
                            let mut ty = quote::format_ident!("PackField{}", idx);
                            ty.set_span(field.ty.span());

                            syn::WherePredicate::Type(syn::PredicateType {
                                lifetimes: None,
                                bounded_ty: syn::Type::Path(syn::TypePath {
                                    qself: None,
                                    path: ty.into(),
                                }),
                                colon_token: Default::default(),
                                bounds: std::iter::once(syn::TypeParamBound::Trait(syn::TraitBound {
                                    paren_token: None,
                                    modifier: syn::TraitBoundModifier::None,
                                    lifetimes: None,
                                    path: syn::Path {
                                        leading_colon: Some(Default::default()),
                                        segments: std::array::IntoIter::new([
                                            syn::PathSegment::from(syn::Ident::new(
                                                "alkahest",
                                                field.span(),
                                            )),
                                            syn::PathSegment {
                                                ident: syn::Ident::new("Pack", field.span()),
                                                arguments: syn::PathArguments::AngleBracketed(
                                                    syn::AngleBracketedGenericArguments {
                                                        colon2_token: None,
                                                        lt_token,
                                                        args: std::iter::once(
                                                            syn::GenericArgument::Type(
                                                                field.ty.clone(),
                                                            ),
                                                        )
                                                        .collect(),
                                                        gt_token,
                                                    },
                                                ),
                                            },
                                        ])
                                        .collect(),
                                    },
                                }))
                                .collect(),
                            })
                        }));
                }
                let (pack_impl_generics, _, pack_where_clause) = pack_generics.split_for_impl();

                

                let packing_fields = variant.fields.iter().enumerate().map(|(idx, field)| {
                    match &field.ident {
                        None => {
                            let member = syn::Member::Unnamed(syn::Index {
                                index: idx as u32,
                                span: field.span(),
                            });
                            quote::quote_spanned!(field.span() => {
                                    let (packed, field_used) = self.#member.pack(offset + used, &mut bytes[used..]);
                                    used += field_used;
                                    packed
                                }
                            )
                        }
                        Some(ident) => quote::quote_spanned!(field.span() => #ident: {
                            let (packed, field_used) = self.#ident.pack(offset + used, &mut bytes[used..]);
                            used += field_used;
                            packed
                        }),
                    }
                });

                let packed_variant_type = quote::format_ident!("{}{}Packed", ident, variant.ident);

                match variant.fields {
                    syn::Fields::Unit => {
                        quote::quote!(
                            #[allow(dead_code)]
                            #vis struct #pack_ident;

                            impl #pack_impl_generics ::alkahest::Pack<#ident #type_generics> for #pack_ident #pack_type_generics #pack_where_clause {
                                #[inline]
                                fn pack(self, offset: usize, bytes: &mut [u8]) -> (#packed_ident #type_generics, usize) {
                                    let mut used = 0;
                                    let packed = #packed_ident {
                                        discriminant: #idx,
                                        variants: #packed_variants_ident {
                                            #variant_ident: #packed_variant_type,
                                        }
                                    };
                                    (packed, used)
                                }
                            }
                        )
                    }
                    syn::Fields::Unnamed(_) => {
                        quote::quote!(
                            #[allow(dead_code)]
                            #vis struct #pack_ident #pack_type_generics ( #( #pack_fields ,)* );

                            impl #pack_impl_generics ::alkahest::Pack<#ident #type_generics> for #pack_ident #pack_type_generics #pack_where_clause {
                                #[inline]
                                fn pack(self, offset: usize, bytes: &mut [u8]) -> (#packed_ident #type_generics, usize) {
                                    let mut used = 0;
                                    let packed = #packed_ident {
                                        discriminant: #idx,
                                        variants: #packed_variants_ident {
                                            #variant_ident: #packed_variant_type ( #( #packing_fields, )* )
                                        }
                                    };
                                    (packed, used)
                                }
                            }
                        )
                        
                    }
                    syn::Fields::Named(_) => {
                        quote::quote!(
                            #[allow(dead_code)]
                            #vis struct #pack_ident #pack_type_generics { #( #pack_fields ,)* }

                            impl #pack_impl_generics ::alkahest::Pack<#ident #type_generics> for #pack_ident #pack_type_generics #pack_where_clause {
                                #[inline]
                                fn pack(self, offset: usize, bytes: &mut [u8]) -> (#packed_ident #type_generics, usize) {
                                    let mut used = 0;
                                    let packed = #packed_ident {
                                        discriminant: #idx,
                                        variants: #packed_variants_ident {
                                            #variant_ident: #packed_variant_type { #( #packing_fields, )* }
                                        }
                                    };
                                    (packed, used)
                                }
                            }
                        )
                    }
                }
            });


            quote::quote!(
                #[allow(dead_code)]
                #vis enum #unpacked_ident #unpacked_generics { #( #unpacked_variants ,)* }

                impl #unpacked_impl_generics ::alkahest::SchemaUnpack<'alkahest> for #ident #type_generics #where_clause {
                    type Unpacked = #unpacked_ident #unpacked_type_generics;
                }

                #(#packed_variants)*

                #[allow(non_snake_case, dead_code)]
                #vis union #packed_variants_ident #generics {
                    #( #vis #packed_variant_idents: #packed_variant_concrete_types ,)*
                }

                impl #impl_generics ::core::clone::Clone for #packed_variants_ident #type_generics #where_clause {
                    #[inline]
                    fn clone(&self) -> Self { *self }
                }

                impl #impl_generics ::core::marker::Copy for #packed_variants_ident #type_generics #where_clause {}

                unsafe impl #impl_generics ::alkahest::Zeroable for #packed_variants_ident #type_generics #where_clause {}
                unsafe impl #impl_generics ::alkahest::Pod for #packed_variants_ident #type_generics #where_clause {}

                #[repr(C, packed)]
                #vis struct #packed_ident #generics {
                    #vis discriminant: u32,
                    #vis variants: #packed_variants_ident #type_generics,
                }

                impl #impl_generics ::core::clone::Clone for #packed_ident #type_generics #where_clause {
                    #[inline]
                    fn clone(&self) -> Self { *self }
                }

                impl #impl_generics ::core::marker::Copy for #packed_ident #type_generics #where_clause {}

                unsafe impl #impl_generics ::alkahest::Zeroable for #packed_ident #type_generics #where_clause {}
                unsafe impl #impl_generics ::alkahest::Pod for #packed_ident #type_generics #where_clause {}

                impl #impl_generics ::alkahest::Schema for #ident #type_generics #where_clause {
                    type Packed = #packed_ident #type_generics;

                    #[inline]
                    fn align() -> usize {
                        1 + (0 #(| #align_masks )*)
                    }

                    #[inline]
                    fn unpack<'alkahest>(packed: #packed_ident #type_generics, bytes: &'alkahest [u8]) -> #unpacked_ident #unpacked_type_generics {
                        match packed.discriminant as usize {
                            #(#unpack_variants,)*
                            _ => panic!("Unknown discriminant")
                        }
                    }
                }

                #(#pack_variants)*
            )
        }
        syn::Data::Struct(data) => {            
            let pack_ident = quote::format_ident!("{}Pack", input.ident);

            let drop_fields =
                data.fields
                    .iter()
                    .enumerate()
                    .map(|(idx, field)| match &field.ident {
                        None => {
                            let member = syn::Member::Unnamed(syn::Index {
                                index: idx as u32,
                                span: field.span(),
                            });
                            quote::quote_spanned!(field.span() => ::core::mem::drop(value.#member))
                        }
                        Some(ident) => {
                            quote::quote_spanned!(field.span() => ::core::mem::drop(value.#ident))
                        }
                    });

            let align_masks = data.fields.iter().map(|field| {
                let ty = &field.ty;
                quote::quote_spanned!(field.span() => (<#ty as ::alkahest::Schema>::align() - 1))
            });

            // Packed
            let packed_fields = data.fields.iter().map(|field| {
                let vis = &field.vis;
                let ty = &field.ty;

                match &field.ident {
                    None => quote::quote_spanned!(field.span() => #vis <#ty as ::alkahest::Schema>::Packed ),
                    Some(ident) => {
                        quote::quote_spanned!(field.span() => #vis #ident: <#ty as ::alkahest::Schema>::Packed )
                    }
                }
            });

            // Unpacked
            let (unpacked_impl_generics, unpacked_type_generics, _) =
                unpacked_generics.split_for_impl();

            let unpacked_fields = data.fields.iter().map(|field| {
                let vis = &field.vis;
                let ty = &field.ty;

                match &field.ident {
                    None => quote::quote_spanned!(field.span() => #vis <#ty as ::alkahest::SchemaUnpack<'alkahest>>::Unpacked ),
                    Some(ident) => {
                        quote::quote_spanned!(field.span() => #vis #ident: <#ty as ::alkahest::SchemaUnpack<'alkahest>>::Unpacked )
                    }
                }
            });


            let unpack_fields = data.fields.iter().enumerate().map(|(idx, field)| {
                let ty = &field.ty;

                match &field.ident {
                    None => {
                        let member = syn::Member::Unnamed(syn::Index { index: idx as u32, span: field.span() });
                        quote::quote_spanned!( field.span() => <#ty as ::alkahest::Schema>::unpack(packed.#member, bytes))
                    }
                    Some(ident) => {
                        quote::quote_spanned!( field.span() => #ident: <#ty as ::alkahest::Schema>::unpack(packed.#ident, bytes))
                    },
                }
            });


            let pack_fields = data.fields.iter().enumerate().map(|(idx, field)| {
                let vis = &field.vis;
                let mut ty = quote::format_ident!("PackField{}", idx);
                ty.set_span(field.ty.span());

                match &field.ident {
                    None => quote::quote_spanned!( field.span() => #vis #ty ),
                    Some(ident) => {
                        quote::quote_spanned!( field.span() => #vis #ident: #ty )
                    }
                }
            });

            let mut pack_generics = input.generics.clone();

            pack_generics
                .params
                .extend((0..data.fields.len()).map(|idx| {
                    syn::GenericParam::Type(syn::TypeParam {
                        ident: quote::format_ident!("PackField{}", idx),
                        attrs: Vec::new(),
                        colon_token: None,
                        bounds: Default::default(),
                        eq_token: None,
                        default: None,
                    })
                }));

            let pack_type_generics = if data.fields.is_empty() {
                syn::Generics::default()
            } else {
                syn::Generics {
                    lt_token: Some(lt_token),
                    params: (0..data.fields.len())
                        .map(|idx| {
                            syn::GenericParam::Type(syn::TypeParam {
                                ident: quote::format_ident!("PackField{}", idx),
                                attrs: Vec::new(),
                                colon_token: None,
                                bounds: Default::default(),
                                eq_token: None,
                                default: None,
                            })
                        })
                        .collect(),
                    gt_token: Some(gt_token),
                    where_clause: None,
                }
            };

            if !data.fields.is_empty() {
                let pack_where_clause =
                    pack_generics
                        .where_clause
                        .get_or_insert_with(|| syn::WhereClause {
                            where_token: Default::default(),
                            predicates: Default::default(),
                        });

                pack_where_clause
                    .predicates
                    .extend(data.fields.iter().enumerate().map(|(idx, field)| {
                        let mut ty = quote::format_ident!("PackField{}", idx);
                        ty.set_span(field.ty.span());

                        syn::WherePredicate::Type(syn::PredicateType {
                            lifetimes: None,
                            bounded_ty: syn::Type::Path(syn::TypePath {
                                qself: None,
                                path: ty.into(),
                            }),
                            colon_token: Default::default(),
                            bounds: std::iter::once(syn::TypeParamBound::Trait(syn::TraitBound {
                                paren_token: None,
                                modifier: syn::TraitBoundModifier::None,
                                lifetimes: None,
                                path: syn::Path {
                                    leading_colon: Some(Default::default()),
                                    segments: std::array::IntoIter::new([
                                        syn::PathSegment::from(syn::Ident::new(
                                            "alkahest",
                                            field.span(),
                                        )),
                                        syn::PathSegment {
                                            ident: syn::Ident::new("Pack", field.span()),
                                            arguments: syn::PathArguments::AngleBracketed(
                                                syn::AngleBracketedGenericArguments {
                                                    colon2_token: None,
                                                    lt_token,
                                                    args: std::iter::once(
                                                        syn::GenericArgument::Type(
                                                            field.ty.clone(),
                                                        ),
                                                    )
                                                    .collect(),
                                                    gt_token,
                                                },
                                            ),
                                        },
                                    ])
                                    .collect(),
                                },
                            }))
                            .collect(),
                        })
                    }));
            }
            let (pack_impl_generics, _, pack_where_clause) = pack_generics.split_for_impl();

            

            let packing_fields = data.fields.iter().enumerate().map(|(idx, field)| {
                match &field.ident {
                    None => {
                        let member = syn::Member::Unnamed(syn::Index {
                            index: idx as u32,
                            span: field.span(),
                        });
                        quote::quote_spanned!(field.span() => {
                                let (packed, field_used) = self.#member.pack(offset + used, &mut bytes[used..]);
                                used += field_used;
                                packed
                            }
                        )
                    }
                    Some(ident) => quote::quote_spanned!(field.span() => #ident: {
                        let (packed, field_used) = self.#ident.pack(offset + used, &mut bytes[used..]);
                        used += field_used;
                        packed
                    }
                ),
                }
            });


            match data.fields {
                syn::Fields::Unit => {
                    quote::quote!(
                        impl ::alkahest::SchemaUnpack<'alkahest> for #ident {
                            type Unpacked = #ident;
                        }

                        impl ::alkahest::Schema for #ident {
                            type Packed = ();

                            #[inline]
                            fn align() -> usize { 1 }

                            #[inline]
                            fn unpack<'alkahest>(packed: (), _bytes: &'alkahest [u8]) -> Self {
                                #ident
                            }
                        }

                        #vis struct #pack_ident;

                        impl ::alkahest::Pack<#ident> for #pack_ident {
                            #[inline]
                            fn pack(self, offset: usize, bytes: &mut [u8]) -> ((), usize) {
                                ((), 0)
                            }
                        }
                    )
                }
                syn::Fields::Unnamed(_) => quote::quote!(
                    #[allow(dead_code)]
                    #vis struct #unpacked_ident #unpacked_generics ( #( #unpacked_fields ,)* );

                    impl #unpacked_impl_generics ::alkahest::SchemaUnpack<'alkahest> for #ident #type_generics #where_clause {
                        type Unpacked = #unpacked_ident #unpacked_type_generics;
                    }

                    #[repr(C, packed)]
                    #[allow(dead_code)]
                    #vis struct #packed_ident #generics ( #( #packed_fields ,)* );

                    impl #impl_generics ::core::clone::Clone for #packed_ident #type_generics #where_clause {
                        #[inline]
                        fn clone(&self) -> Self { *self }
                    }

                    impl #impl_generics ::core::marker::Copy for #packed_ident #type_generics #where_clause {}

                    unsafe impl #impl_generics ::alkahest::Zeroable for #packed_ident #type_generics #where_clause {}
                    unsafe impl #impl_generics ::alkahest::Pod for #packed_ident #type_generics #where_clause {}

                    impl #impl_generics ::alkahest::Schema for #ident #type_generics #where_clause {
                        type Packed = #packed_ident #type_generics;

                        #[inline]
                        fn align() -> usize {
                            #[allow(dead_code)]
                            fn drop_fields #generics (value: #ident #type_generics) {
                                #( #drop_fields ; )*
                            }

                            1 + (0 #(| #align_masks )*)
                        }

                        #[inline]
                        fn unpack<'alkahest>(packed: #packed_ident, bytes: &'alkahest [u8]) -> #unpacked_ident #unpacked_type_generics {
                            #unpacked_ident (
                                #(#unpack_fields, )*
                            )
                        }
                    }

                    #[allow(dead_code)]
                    #vis struct #pack_ident #pack_type_generics ( #( #pack_fields ,)* );

                    impl #pack_impl_generics ::alkahest::Pack<#ident #type_generics> for #pack_ident #pack_type_generics #pack_where_clause {
                        #[inline]
                        fn pack(self, offset: usize, bytes: &mut [u8]) -> (#packed_ident #type_generics, usize) {
                            let mut used = 0;
                            let packed = #packed_ident (
                                #( #packing_fields, )*
                            );
                            (packed, used)
                        }
                    }
                ),
                syn::Fields::Named(_) => quote::quote!(
                    #[allow(dead_code)]
                    #vis struct #unpacked_ident #unpacked_generics { #( #unpacked_fields ,)* }

                    impl #unpacked_impl_generics ::alkahest::SchemaUnpack<'alkahest> for #ident #type_generics #where_clause {
                        type Unpacked = #unpacked_ident #unpacked_type_generics;
                    }

                    #[repr(C, packed)]
                    #[allow(dead_code)]
                    #vis struct #packed_ident #generics { #( #packed_fields ,)* }

                    impl #impl_generics ::core::clone::Clone for #packed_ident #type_generics #where_clause {
                        #[inline]
                        fn clone(&self) -> Self { *self }
                    }

                    impl #impl_generics ::core::marker::Copy for #packed_ident #type_generics #where_clause {}

                    unsafe impl #impl_generics ::alkahest::Zeroable for #packed_ident #type_generics #where_clause {}
                    unsafe impl #impl_generics ::alkahest::Pod for #packed_ident #type_generics #where_clause {}

                    impl #impl_generics ::alkahest::Schema for #ident #type_generics #where_clause {
                        type Packed = #packed_ident #type_generics;

                        #[inline]
                        fn align() -> usize {
                            #[allow(dead_code)]
                            fn drop_fields #generics (value: #ident #type_generics) {
                                #( #drop_fields ; )*
                            }

                            1 + (0 #(| #align_masks )*)
                        }

                        #[inline]
                        fn unpack<'alkahest>(packed: #packed_ident #type_generics, bytes: &'alkahest [u8]) -> #unpacked_ident #unpacked_type_generics {
                            #unpacked_ident {
                                #(#unpack_fields, )*
                            }
                        }
                    }

                    #[allow(dead_code)]
                    #vis struct #pack_ident #pack_type_generics { #( #pack_fields ,)* }

                    impl #pack_impl_generics ::alkahest::Pack<#ident #type_generics> for #pack_ident #pack_type_generics #pack_where_clause {
                        #[inline]
                        fn pack(self, offset: usize, bytes: &mut [u8]) -> (#packed_ident #type_generics, usize) {
                            let mut used = 0;
                            let packed = #packed_ident {
                                #( #packing_fields, )*
                            };
                            (packed, used)
                        }
                    }
                ),
            }
        }
        syn::Data::Union(data) => {
            return Err(syn::Error::new_spanned(
                data.union_token,
                "Unions are not supported by `Schema` derive macro",
            ))
        }
    };

    Ok(result)
}
