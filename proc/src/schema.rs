use {proc_macro2::TokenStream, std::convert::TryFrom, syn::spanned::Spanned};

pub fn derive_schema(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    for param in &input.generics.params {
        if let syn::GenericParam::Lifetime(lifetime) = param {
            return Err(syn::Error::new_spanned(
                lifetime,
                "Schema derive macro does not support structures with lifetime parameters",
            ));
        }
    }

    let cfg = crate::AlkahestConfig::from_input(&input)?;
    let derive_owned = cfg.derive_owned;
    let schema_bounds = cfg.schema_bounds;
    let owned_bounds = cfg.owned_bounds;

    let vis = &input.vis;
    let ident = &input.ident;
    let packed_ident = quote::format_ident!("{}Packed", input.ident);
    let unpacked_ident = quote::format_ident!("{}Unpacked", input.ident);

    let mut schema_generics = input.generics.clone();

    match &mut schema_generics.where_clause {
        Some(where_clause) => where_clause.predicates.extend(schema_bounds.predicates),
        none => *none = Some(schema_bounds),
    }

    let mut schema_unpack_generics = schema_generics.clone();
    schema_unpack_generics.params.push(syn::parse_quote!('a));

    let (schema_impl_generics, schema_type_generics, schema_where_clause) =
        schema_generics.split_for_impl();

    let (schema_unpack_impl_generics, schema_unpack_type_generics, schema_unpack_where_clause) =
        schema_unpack_generics.split_for_impl();

    let mut owned_generics = input.generics.clone();
    match &mut owned_generics.where_clause {
        Some(where_clause) => where_clause.predicates.extend(owned_bounds.predicates),
        none => *none = Some(owned_bounds),
    }

    let (owned_impl_generics, owned_type_generics, owned_where_clause) =
        owned_generics.split_for_impl();

    let result = match input.data {
        syn::Data::Enum(data) => {
            let no_fields = data.variants.iter().all(|v| v.fields.is_empty());

            let packed_variants_ident = quote::format_ident!("{}PackedVariants", input.ident);

            let unpacked_impl_generics = if no_fields {
                &schema_impl_generics
            } else {
                &schema_unpack_impl_generics
            };

            let unpacked_type_generics = if no_fields {
                &schema_type_generics
            } else {
                &schema_unpack_type_generics
            };

            let unpacked_where_clause = if no_fields {
                &schema_where_clause
            } else {
                &schema_unpack_where_clause
            };

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
                    let bounds: syn::punctuated::Punctuated<_, _> = std::iter::once::<syn::TypeParamBound>(syn::parse_quote!(::alkahest::Pod)).collect();

                    syn::Generics {
                        lt_token: Some(Default::default()),
                        params: (0..variant.fields.len())
                            .map(|idx| {
                                syn::GenericParam::Type(syn::TypeParam {
                                    ident: quote::format_ident!("ALKAHEST_T{}", idx),
                                    attrs: Vec::new(),
                                    colon_token: None,
                                    bounds: bounds.clone(),
                                    eq_token: None,
                                    default: None,
                                })
                            })
                            .collect(),
                        gt_token: Some(Default::default()),
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
                            let ty = quote::format_ident!("ALKAHEST_T{}", idx);
                            quote::quote_spanned!(field.span() => pub #ty )
                        });

                        quote::quote_spanned!(variant.span() =>
                            #[repr(C, packed)] #vis struct #packed_variant_ident #packed_variant_type_generics ( #(#packed_fields,)* );

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
                            let ty = quote::format_ident!("ALKAHEST_T{}", idx);
                            let ident = field.ident.as_ref().unwrap();
                            quote::quote_spanned!(field.span() => pub #ident: #ty )
                        });

                        quote::quote_spanned!(variant.span() =>
                            #[repr(C, packed)] #vis struct #packed_variant_ident #packed_variant_type_generics { #(#packed_fields,)* }

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

            let packed_variant_idents = data.variants.iter().map(|variant| &variant.ident);

            let packed_variant_concrete_types = data.variants.iter().map(|variant| -> syn::Type {
                let ident = quote::format_ident!("{}{}Packed", ident, variant.ident);

                let args = variant.fields.iter().map(|field| {
                    syn::GenericArgument::Type({
                        let ty = &field.ty;
                        syn::parse_quote!(<#ty as ::alkahest::Schema>::Packed)
                    })
                });

                if variant.fields.is_empty() {
                    syn::parse_quote!(#ident)
                } else {
                    syn::parse_quote!(#ident <#(#args),*> )
                }
            });

            let unpacked_variants = data.variants.iter().map(|variant| {
                let variant_ident= &variant.ident;

                match &variant.fields {
                    syn::Fields::Unit => quote::quote_spanned!(variant.span() => #variant_ident),
                    syn::Fields::Unnamed(fields) => {
                        let unpacked_fields = fields.unnamed.iter().map(|field| {
                            let ty = &field.ty;
                            quote::quote_spanned!(field.span() => <#ty as ::alkahest::SchemaUnpack<'a>>::Unpacked )
                        });

                        quote::quote_spanned!(variant.span() => #variant_ident ( #(#unpacked_fields,)* ))
                    }
                    syn::Fields::Named(fields) => {
                        let unpacked_fields = fields.named.iter().map(|field| {
                            let ty = &field.ty;
                            let ident = field.ident.as_ref().unwrap();
                            quote::quote_spanned!(field.span() => #ident: <#ty as ::alkahest::SchemaUnpack<'a>>::Unpacked )
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
                            quote::quote_spanned!( field.span() => <#ty as ::alkahest::Schema>::unpack(unsafe { packed.variants.#variant_ident.#member }, bytes))
                        });
                        quote::quote_spanned!(variant.span() => #idx => {
                            #unpacked_ident::#variant_ident ( #( #unpack_fields, )* )
                        })
                    }
                    syn::Fields::Named(fields) => {
                        let unpack_fields = fields.named.iter().map(|field| {
                            let ty = &field.ty;
                            let ident = field.ident.as_ref().unwrap();
                            quote::quote_spanned!( field.span() => #ident: <#ty as ::alkahest::Schema>::unpack(unsafe { packed.variants.#variant_ident.#ident }, bytes))
                        });
                        quote::quote_spanned!(variant.span() => #idx => {
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
                    let mut ty = quote::format_ident!("ALKAHEST_T{}", idx);
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
                        lt_token: Some(Default::default()),
                        params: (0..variant.fields.len())
                            .map(|idx| -> syn::GenericParam {
                                let ident = quote::format_ident!("ALKAHEST_T{}", idx);
                                syn::parse_quote!(#ident)
                            })
                            .collect(),
                        gt_token: Some(Default::default()),
                        where_clause: None,
                    }
                };

                let mut pack_generics = schema_generics.clone();

                pack_generics
                    .params
                    .extend((0..variant.fields.len()).map(|idx| {
                        syn::GenericParam::Type(syn::TypeParam {
                            ident: quote::format_ident!("ALKAHEST_T{}", idx),
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
                        .extend(variant.fields.iter().enumerate().map(|(idx, field)| -> syn::WherePredicate {
                            let mut ty = quote::format_ident!("ALKAHEST_T{}", idx);
                            ty.set_span(field.ty.span());

                            let field_ty = &field.ty;

                            syn::parse_quote!(#ty: ::alkahest::Pack<#field_ty>)
                        }));
                }

                let (pack_impl_generics, _, pack_where_clause) = pack_generics.split_for_impl();

                let packing_fields = variant.fields.iter().enumerate().map(|(idx, field)| {
                    let ty = &field.ty;
                    match &field.ident {
                        None => {
                            let member = syn::Member::Unnamed(syn::Index {
                                index: idx as u32,
                                span: field.span(),
                            });
                            quote::quote_spanned!(field.span() => {
                                    let align_mask = <#ty as ::alkahest::Schema>::align() - 1;
                                    debug_assert_eq!(offset & align_mask, 0, "Offset is not aligned to {}", align_mask + 1);
                                    let aligned = (used + align_mask) & !align_mask;
                                    let (packed, field_used) = self.#member.pack(offset + aligned, &mut bytes[aligned..]);
                                    used = aligned + field_used;
                                    packed
                                }
                            )
                        }
                        Some(ident) => quote::quote_spanned!(field.span() => #ident: {
                            let align_mask = <#ty as ::alkahest::Schema>::align() - 1;
                            debug_assert_eq!(offset & align_mask, 0, "Offset is not aligned to {}", align_mask + 1);
                            let aligned = (used + align_mask) & !align_mask;
                            let (packed, field_used) = self.#ident.pack(offset + aligned, &mut bytes[aligned..]);
                            used = aligned + field_used;
                            packed
                        }),
                    }
                });
                
                let packed_variant_type = quote::format_ident!("{}{}Packed", ident, variant.ident);

                match variant.fields {
                    syn::Fields::Unit => {
                        quote::quote!(
                            #[derive(Clone, Copy, Debug)]
                            #[allow(dead_code)]
                            #vis struct #pack_ident;

                            impl #pack_impl_generics ::alkahest::Pack<#ident #schema_type_generics> for #pack_ident #pack_where_clause {
                                #[inline]
                                fn pack(self, offset: usize, bytes: &mut [u8]) -> (#packed_ident #schema_type_generics, usize) {
                                    let align_mask = <#ident #schema_type_generics as ::alkahest::Schema>::align() - 1;
                                    debug_assert_eq!(bytes.as_ptr() as usize & align_mask, 0, "Output is not aligned to {}", align_mask + 1);
                                    debug_assert_eq!(offset & align_mask, 0, "Offset is not aligned to {}", align_mask + 1);

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
                            #[derive(Clone, Copy, Debug)]
                            #[allow(dead_code)]
                            #vis struct #pack_ident #pack_type_generics ( #( #pack_fields ,)* );

                            impl #pack_impl_generics ::alkahest::Pack<#ident #schema_type_generics> for #pack_ident #pack_type_generics #pack_where_clause {
                                #[inline]
                                fn pack(self, offset: usize, bytes: &mut [u8]) -> (#packed_ident #schema_type_generics, usize) {
                                    let align_mask = <#ident #schema_type_generics as ::alkahest::Schema>::align() - 1;
                                    debug_assert_eq!(bytes.as_ptr() as usize & align_mask, 0, "Output is not aligned to {}", align_mask + 1);
                                    debug_assert_eq!(offset & align_mask, 0, "Offset is not aligned to {}", align_mask + 1);

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
                            #[derive(Clone, Copy, Debug)]
                            #[allow(dead_code)]
                            #vis struct #pack_ident #pack_type_generics { #( #pack_fields ,)* }

                            impl #pack_impl_generics ::alkahest::Pack<#ident #schema_type_generics> for #pack_ident #pack_type_generics #pack_where_clause {
                                #[inline]
                                fn pack(self, offset: usize, bytes: &mut [u8]) -> (#packed_ident #schema_type_generics, usize) {  
                                    let align_mask = <#ident #schema_type_generics as ::alkahest::Schema>::align() - 1;
                                    debug_assert_eq!(bytes.as_ptr() as usize & align_mask, 0, "Output is not aligned to {}", align_mask + 1);
                                    debug_assert_eq!(offset & align_mask, 0, "Offset is not aligned to {}", align_mask + 1);
                                  
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

            let owned = if derive_owned {
                let variants_to_owned = data.variants.iter().map(|variant| {
                    let variant_ident = &variant.ident;

                    match &variant.fields {
                        syn::Fields::Unit => quote::quote!(#unpacked_ident :: #variant_ident => #ident :: #variant_ident),
                        syn::Fields::Unnamed(fields) => {
                            let fields = fields.unnamed.iter().enumerate().map(|(idx, field)| {
                                let ident = quote::format_ident!("a{}", idx);
                                quote::quote_spanned!(field.span() => #ident)
                            }).collect::<Vec<_>>();

                            quote::quote!(#unpacked_ident :: #variant_ident ( #(#fields),* ) => #ident :: #variant_ident ( #(::alkahest::SchemaOwned::to_owned_schema(#fields)),* ))
                        }
                        syn::Fields::Named(fields) => {
                            let fields = fields.named.iter().map(|field| {
                                let ident = field.ident.as_ref().unwrap();
                                quote::quote_spanned!(field.span() => #ident)
                            }).collect::<Vec<_>>();

                            quote::quote!(#unpacked_ident :: #variant_ident { #(#fields),* } => #ident :: #variant_ident { #(#fields: ::alkahest::SchemaOwned::to_owned_schema(#fields)),* })
                        }
                    }
                });

                quote::quote!(
                    impl #owned_impl_generics alkahest::SchemaOwned for #ident #owned_type_generics #owned_where_clause {
                        fn to_owned_schema<'a>(unpacked: #unpacked_ident #unpacked_type_generics) -> Self {
                            match unpacked {
                                #( #variants_to_owned ),*
                            }
                        }
                    }
                )
            } else {
                Default::default()
            };

            quote::quote!(
                #[allow(dead_code)]
                #vis enum #unpacked_ident #unpacked_impl_generics #unpacked_where_clause  { #( #unpacked_variants ,)* }

                impl #schema_unpack_impl_generics ::alkahest::SchemaUnpack<'a> for #ident #schema_type_generics #schema_where_clause {
                    type Unpacked = #unpacked_ident #unpacked_type_generics;
                }

                #(#packed_variants)*

                #[allow(non_snake_case, dead_code)]
                #vis union #packed_variants_ident #schema_impl_generics #schema_where_clause {
                    pub _alkahest_packed_enum_uninit: (),
                    #( #vis #packed_variant_idents: #packed_variant_concrete_types ,)*
                }

                impl #schema_impl_generics ::core::clone::Clone for #packed_variants_ident #schema_type_generics #schema_where_clause {
                    #[inline]
                    fn clone(&self) -> Self { *self }
                }

                impl #schema_impl_generics ::core::marker::Copy for #packed_variants_ident #schema_type_generics #schema_where_clause {}

                unsafe impl #schema_impl_generics ::alkahest::Zeroable for #packed_variants_ident #schema_type_generics #schema_where_clause {}
                unsafe impl #schema_impl_generics ::alkahest::Pod for #packed_variants_ident #schema_type_generics #schema_where_clause {}

                #[repr(C, packed)]
                #vis struct #packed_ident #schema_impl_generics #schema_where_clause {
                    #vis discriminant: u32,
                    #vis variants: #packed_variants_ident #schema_type_generics,
                }

                impl #schema_impl_generics ::core::clone::Clone for #packed_ident #schema_type_generics #schema_where_clause {
                    #[inline]
                    fn clone(&self) -> Self { *self }
                }

                impl #schema_impl_generics ::core::marker::Copy for #packed_ident #schema_type_generics #schema_where_clause {}

                unsafe impl #schema_impl_generics ::alkahest::Zeroable for #packed_ident #schema_type_generics #schema_where_clause {}
                unsafe impl #schema_impl_generics ::alkahest::Pod for #packed_ident #schema_type_generics #schema_where_clause {}

                impl #schema_impl_generics ::alkahest::Schema for #ident #schema_type_generics #schema_where_clause {
                    type Packed = #packed_ident #schema_type_generics;

                    #[inline]
                    fn align() -> usize {
                        1 + (0 #(| #align_masks )*)
                    }

                    #[inline]
                    fn unpack<'a>(packed: #packed_ident #schema_type_generics, bytes: &'a [u8]) -> #unpacked_ident #unpacked_type_generics {
                        match packed.discriminant as usize {
                            #(#unpack_variants,)*
                            _ => panic!("Unknown discriminant")
                        }
                    }
                }

                #(#pack_variants)*

                #owned
            )
        }
        syn::Data::Struct(data) => {
            let pack_ident = quote::format_ident!("{}Pack", input.ident);

            let no_fields = data.fields.is_empty();

            let unpacked_impl_generics = if no_fields {
                &schema_impl_generics
            } else {
                &schema_unpack_impl_generics
            };

            let unpacked_type_generics = if no_fields {
                &schema_type_generics
            } else {
                &schema_unpack_type_generics
            };

            let unpacked_where_clause = if no_fields {
                &schema_where_clause
            } else {
                &schema_unpack_where_clause
            };

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

            let unpacked_fields = data.fields.iter().map(|field| {
                let vis = &field.vis;
                let ty = &field.ty;

                match &field.ident {
                    None => quote::quote_spanned!(field.span() => #vis <#ty as ::alkahest::SchemaUnpack<'a>>::Unpacked ),
                    Some(ident) => {
                        quote::quote_spanned!(field.span() => #vis #ident: <#ty as ::alkahest::SchemaUnpack<'a>>::Unpacked )
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
                let mut ty = quote::format_ident!("ALKAHEST_T{}", idx);
                ty.set_span(field.ty.span());

                match &field.ident {
                    None => quote::quote_spanned!( field.span() => #vis #ty ),
                    Some(ident) => {
                        quote::quote_spanned!( field.span() => #vis #ident: #ty )
                    }
                }
            });

            let pack_type_generics = if data.fields.is_empty() {
                syn::Generics::default()
            } else {
                syn::Generics {
                    lt_token: Some(Default::default()),
                    params: (0..data.fields.len())
                        .map(|idx| -> syn::GenericParam {
                            let ident = quote::format_ident!("ALKAHEST_T{}", idx);
                            syn::parse_quote!(#ident)
                        })
                        .collect(),
                    gt_token: Some(Default::default()),
                    where_clause: None,
                }
            };

            let mut pack_generics = schema_generics.clone();

            pack_generics
                .params
                .extend((0..data.fields.len()).map(|idx| {
                    syn::GenericParam::Type(syn::TypeParam {
                        ident: quote::format_ident!("ALKAHEST_T{}", idx),
                        attrs: Vec::new(),
                        colon_token: None,
                        bounds: Default::default(),
                        eq_token: None,
                        default: None,
                    })
                }));

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
                    .extend(data.fields.iter().enumerate().map(
                        |(idx, field)| -> syn::WherePredicate {
                            let mut ty = quote::format_ident!("ALKAHEST_T{}", idx);
                            ty.set_span(field.ty.span());

                            let field_ty = &field.ty;

                            syn::parse_quote!(#ty: ::alkahest::Pack<#field_ty>)
                        },
                    ));
            }

            let (pack_impl_generics, _, pack_where_clause) = pack_generics.split_for_impl();

            let packing_fields = data.fields.iter().enumerate().map(|(idx, field)| {
                let ty = &field.ty;
                match &field.ident {
                    None => {
                        let member = syn::Member::Unnamed(syn::Index {
                            index: idx as u32,
                            span: field.span(),
                        });
                        quote::quote_spanned!(field.span() => {
                                let align_mask = <#ty as ::alkahest::Schema>::align() - 1;
                                debug_assert_eq!(offset & align_mask, 0, "Offset is not aligned to {}", align_mask + 1);
                                let aligned = (used + align_mask) & !align_mask;
                                let (packed, field_used) = self.#member.pack(offset + aligned, &mut bytes[aligned..]);
                                used = aligned + field_used;
                                packed
                            }
                        )
                    }
                    Some(ident) => quote::quote_spanned!(field.span() => #ident: {
                        let align_mask = <#ty as ::alkahest::Schema>::align() - 1;
                        debug_assert_eq!(offset & align_mask, 0, "Offset is not aligned to {}", align_mask + 1);
                        let aligned = (used + align_mask) & !align_mask;
                        let (packed, field_used) = self.#ident.pack(offset + aligned, &mut bytes[aligned..]);
                        used = aligned + field_used;
                        packed
                    }
                ),
                }
            });

            let owned = if derive_owned {
                match &data.fields {
                    syn::Fields::Unit => quote::quote!(
                        impl alkahest::SchemaOwned for #ident {
                            fn to_owned_schema<'a>(unpacked: #ident) -> Self {
                                unpacked
                            }
                        }
                    ),
                    syn::Fields::Unnamed(fields) => {
                        let fields_to_owned = fields.unnamed.iter().enumerate().map(|(idx, field)| {
                            let member = syn::Member::Unnamed(syn::Index { index: idx as u32, span: field.span() });
                            quote::quote_spanned!(field.span() => ::alkahest::SchemaOwned::to_owned_schema(unpacked.#member))
                        });

                        quote::quote!(
                            impl #owned_impl_generics alkahest::SchemaOwned for #ident #owned_type_generics #owned_where_clause {
                                fn to_owned_schema<'a>(unpacked: #unpacked_ident #unpacked_type_generics) -> Self {
                                    #ident ( #(#fields_to_owned),* )
                                }
                            }
                        )
                    }
                    syn::Fields::Named(fields) => {
                        let fields_to_owned = fields.named.iter().map(|field| {
                            let ident = field.ident.as_ref().unwrap();
                            quote::quote_spanned!(field.span() => #ident: ::alkahest::SchemaOwned::to_owned_schema(unpacked.#ident))
                        });

                        quote::quote!(
                            impl #owned_impl_generics alkahest::SchemaOwned for #ident #owned_type_generics #owned_where_clause {
                                fn to_owned_schema<'a>(unpacked: #unpacked_ident #unpacked_type_generics) -> Self {
                                    #ident { #(#fields_to_owned),* }
                                }
                            }
                        )
                    }
                }
            } else {
                Default::default()
            };

            match data.fields {
                syn::Fields::Unit => {
                    quote::quote!(
                        impl ::alkahest::SchemaUnpack<'a> for #ident {
                            type Unpacked = #ident;
                        }

                        impl ::alkahest::Schema for #ident {
                            type Packed = ();

                            #[inline]
                            fn align() -> usize { 1 }

                            #[inline]
                            fn unpack<'a>(packed: (), _bytes: &'a [u8]) -> Self {
                                #ident
                            }
                        }

                        #[derive(Clone, Copy, Debug)]
                        #vis struct #pack_ident;

                        impl ::alkahest::Pack<#ident> for #pack_ident {
                            #[inline]
                            fn pack(self, offset: usize, bytes: &mut [u8]) -> ((), usize) {
                                ((), 0)
                            }
                        }

                        #owned
                    )
                }
                syn::Fields::Unnamed(_) => quote::quote!(
                    #[allow(dead_code)]
                    #vis struct #unpacked_ident #unpacked_impl_generics #unpacked_where_clause  ( #( #unpacked_fields ,)* );

                    impl #schema_unpack_impl_generics ::alkahest::SchemaUnpack<'a> for #ident #schema_type_generics #schema_where_clause {
                        type Unpacked = #unpacked_ident #unpacked_type_generics;
                    }

                    #[repr(C, packed)]
                    #vis struct #packed_ident #schema_impl_generics #schema_where_clause ( #( #packed_fields ,)* );

                    impl #schema_impl_generics ::core::clone::Clone for #packed_ident #schema_type_generics #schema_where_clause {
                        #[inline]
                        fn clone(&self) -> Self { *self }
                    }

                    impl #schema_impl_generics ::core::marker::Copy for #packed_ident #schema_type_generics #schema_where_clause {}

                    unsafe impl #schema_impl_generics ::alkahest::Zeroable for #packed_ident #schema_type_generics #schema_where_clause {}
                    unsafe impl #schema_impl_generics ::alkahest::Pod for #packed_ident #schema_type_generics #schema_where_clause {}

                    impl #schema_impl_generics ::alkahest::Schema for #ident #schema_type_generics #schema_where_clause {
                        type Packed = #packed_ident #schema_type_generics;

                        #[inline]
                        fn align() -> usize {
                            1 + (0 #(| #align_masks )*)
                        }

                        #[inline]
                        fn unpack<'a>(packed: #packed_ident #schema_type_generics, bytes: &'a [u8]) -> #unpacked_ident #unpacked_type_generics {
                            #unpacked_ident (
                                #(#unpack_fields, )*
                            )
                        }
                    }

                    #[derive(Clone, Copy, Debug)]
                    #[allow(dead_code)]
                    #vis struct #pack_ident #pack_type_generics ( #( #pack_fields ,)* );

                    impl #pack_impl_generics ::alkahest::Pack<#ident #schema_type_generics> for #pack_ident #pack_type_generics #pack_where_clause {
                        #[inline]
                        fn pack(self, offset: usize, bytes: &mut [u8]) -> (#packed_ident #schema_type_generics, usize) {
                            let align_mask = <#ident #schema_type_generics as ::alkahest::Schema>::align() - 1;
                            debug_assert_eq!(bytes.as_ptr() as usize & align_mask, 0, "Output is not aligned to {}", align_mask + 1);
                            debug_assert_eq!(offset & align_mask, 0, "Offset is not aligned to {}", align_mask + 1);

                            let mut used = 0;
                            let packed = #packed_ident (
                                #( #packing_fields, )*
                            );
                            (packed, used)
                        }
                    }

                    #owned
                ),
                syn::Fields::Named(_) => quote::quote!(
                    #[allow(dead_code)]
                    #vis struct #unpacked_ident #unpacked_impl_generics #unpacked_where_clause { #( #unpacked_fields ,)* }

                    impl #schema_unpack_impl_generics ::alkahest::SchemaUnpack<'a> for #ident #schema_type_generics #schema_where_clause {
                        type Unpacked = #unpacked_ident #unpacked_type_generics;
                    }

                    #[repr(C, packed)]
                    #vis struct #packed_ident #schema_impl_generics #schema_where_clause { #( #packed_fields ,)* }

                    impl #schema_impl_generics ::core::clone::Clone for #packed_ident #schema_type_generics #schema_where_clause {
                        #[inline]
                        fn clone(&self) -> Self { *self }
                    }

                    impl #schema_impl_generics ::core::marker::Copy for #packed_ident #schema_type_generics #schema_where_clause {}

                    unsafe impl #schema_impl_generics ::alkahest::Zeroable for #packed_ident #schema_type_generics #schema_where_clause {}
                    unsafe impl #schema_impl_generics ::alkahest::Pod for #packed_ident #schema_type_generics #schema_where_clause {}

                    impl #schema_impl_generics ::alkahest::Schema for #ident #schema_type_generics #schema_where_clause {
                        type Packed = #packed_ident #schema_type_generics;

                        #[inline]
                        fn align() -> usize {
                            1 + (0 #(| #align_masks )*)
                        }

                        #[inline]
                        fn unpack<'a>(packed: #packed_ident #schema_type_generics, bytes: &'a [u8]) -> #unpacked_ident #unpacked_type_generics {
                            #unpacked_ident {
                                #(#unpack_fields, )*
                            }
                        }
                    }

                    #[derive(Clone, Copy, Debug)]
                    #[allow(dead_code)]
                    #vis struct #pack_ident #pack_type_generics { #( #pack_fields ,)* }

                    impl #pack_impl_generics ::alkahest::Pack<#ident #schema_type_generics> for #pack_ident #pack_type_generics #pack_where_clause {
                        #[inline]
                        fn pack(self, offset: usize, bytes: &mut [u8]) -> (#packed_ident #schema_type_generics, usize) {
                            let align_mask = <#ident #schema_type_generics as ::alkahest::Schema>::align() - 1;
                            debug_assert_eq!(bytes.as_ptr() as usize & align_mask, 0, "Output is not aligned to {}", align_mask + 1);
                            debug_assert_eq!(offset & align_mask, 0, "Offset is not aligned to {}", align_mask + 1);

                            let mut used = 0;
                            let packed = #packed_ident {
                                #( #packing_fields, )*
                            };
                            (packed, used)
                        }
                    }

                    #owned
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
