use {proc_macro2::TokenStream, syn::spanned::Spanned};

pub fn derive_schema(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    let result = match input.data {
        syn::Data::Enum(_) => unimplemented!(),
        syn::Data::Struct(data) => {
            for param in &input.generics.params {
                if let syn::GenericParam::Lifetime(lifetime) = param {
                    return Err(syn::Error::new_spanned(
                        lifetime,
                        "Schema derive macro does not support structures with lifetime parameters",
                    ));
                }
            }

            let ident = &input.ident;

            let packed_ident = quote::format_ident!("{}Packed", input.ident);
            let unpacked_ident = quote::format_ident!("{}Unpacked", input.ident);
            let pack_ident = quote::format_ident!("{}Pack", input.ident);

            let fields_are_named = std::matches!(data.fields, syn::Fields::Named(_));

            let packed_fields = data.fields.iter().map(|field| {
                let ty = &field.ty;

                match &field.ident {
                    None => quote::quote_spanned!(field.span() => <#ty as alkahest::Schema>::Packed ),
                    Some(ident) => {
                        quote::quote_spanned!(field.span() => #ident: <#ty as alkahest::Schema>::Packed )
                    }
                }
            });

            let unpacked_fields = data.fields.iter().map(|field| {
                let ty = &field.ty;

                match &field.ident {
                    None => quote::quote_spanned!(field.span() => <#ty as alkahest::SchemaUnpack<'alkahest>>::Unpacked ),
                    Some(ident) => {
                        quote::quote_spanned!(field.span() => #ident: <#ty as alkahest::SchemaUnpack<'alkahest>>::Unpacked )
                    }
                }
            });

            let pack_fields = data.fields.iter().enumerate().map(|(idx, field)| {
                let mut ty = quote::format_ident!("PackField{}", idx);
                ty.set_span(field.span());

                match &field.ident {
                    None => quote::quote_spanned!( field.span() => #ty ),
                    Some(ident) => {
                        quote::quote_spanned!( field.span() => #ident: #ty )
                    }
                }
            });

            let generics = &input.generics;
            let mut unpacked_generics = input.generics.clone();
            let mut pack_generics = input.generics.clone();

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
                        ty.set_span(field.span());

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

            let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
            let (unpacked_impl_generics, unpacked_type_generics, _) =
                unpacked_generics.split_for_impl();
            let (pack_impl_generics, _, pack_where_clause) = pack_generics.split_for_impl();

            let align_masks = data.fields.iter().map(|field| {
                let ty = &field.ty;
                quote::quote_spanned!(field.span() => (<#ty as Schema>::align() - 1))
            });

            let unpack_fields = data.fields.iter().enumerate().map(|(i, field)| {
                let ty = &field.ty;
                let member = match &field.ident {
                    None => syn::Member::Unnamed(syn::Index {
                        index: i as u32,
                        span: field.span(),
                    }),
                    Some(ident) => syn::Member::Named(ident.clone()),
                };

                if fields_are_named {
                    quote::quote_spanned!( field.span() => #member: <#ty as Schema>::unpack(packed.#member, bytes)
                    )
                } else {
                    quote::quote_spanned!( field.span() => <#ty as Schema>::unpack(packed.#member, bytes))
                }
            });

            let packing_fields = data.fields.iter().enumerate().map(|(idx, field)| {
                let member = match &field.ident {
                    None => syn::Member::Unnamed(syn::Index {
                        index: idx as u32,
                        span: field.span(),
                    }),
                    Some(ident) => syn::Member::Named(ident.clone()),
                };

                if fields_are_named {
                    quote::quote_spanned!(field.span() => #member: {
                            let (packed, field_used) = self.#member.pack(offset + used, &mut bytes[used..]);
                            used += field_used;
                            packed
                        }
                    )
                } else {
                    quote::quote_spanned!(field.span() => {
                            let (packed, field_used) = self.#member.pack(offset + used, &mut bytes[used..]);
                            used += field_used;
                            packed
                        }
                    )
                }
            });

            let drop_fields = data.fields.iter().enumerate().map(|(idx, field)| {
                let member = match &field.ident {
                    None => syn::Member::Unnamed(syn::Index {
                        index: idx as u32,
                        span: field.span(),
                    }),
                    Some(ident) => syn::Member::Named(ident.clone()),
                };

                quote::quote_spanned!(field.span() => ::core::mem::drop(value.#member))
            });

            let vis = &input.vis;

            match data.fields {
                syn::Fields::Unit => {
                    quote::quote!(
                        impl alkahest::SchemaUnpack<'alkahest> for #ident {
                            type Unpacked = #ident;
                        }

                        impl alkahest::Schema for #ident {
                            type Packed = ();

                            fn align() -> usize { 1 }

                            fn unpack<'alkahest>(packed: (), _bytes: &'alkahest [u8]) -> Self {
                                #ident
                            }
                        }

                        pub struct #pack_ident;

                        impl alkahest::Pack<#ident> for #pack_ident {
                            fn pack(self, offset: usize, bytes: &mut [u8]) -> ((), usize) {
                                ((), 0)
                            }
                        }
                    )
                }
                syn::Fields::Unnamed(_) => quote::quote!(
                    struct #unpacked_ident #unpacked_generics ( #( #unpacked_fields ,)* );

                    impl #unpacked_impl_generics alkahest::SchemaUnpack<'alkahest> for #ident #type_generics #where_clause {
                        type Unpacked = #unpacked_ident #unpacked_type_generics;
                    }

                    #[derive(Clone, Copy)]
                    #[repr(C, packed)]
                    struct #packed_ident #generics ( #( #packed_fields ,)* );

                    unsafe impl #impl_generics alkahest::Zeroable for #packed_ident #type_generics #where_clause {}
                    unsafe impl #impl_generics alkahest::Pod for #packed_ident #type_generics #where_clause {}

                    impl #impl_generics alkahest::Schema for #ident #type_generics #where_clause {
                        type Packed = #packed_ident #type_generics;

                        fn align() -> usize {
                            #[allow(dead_code)]
                            fn drop_fields(value: #ident) {
                                #( #drop_fields ; )*
                            }

                            1 + (0 #(| #align_masks )*)
                        }

                        fn unpack<'alkahest>(packed: #packed_ident, bytes: &'alkahest [u8]) -> #unpacked_ident #unpacked_type_generics {
                            #unpacked_ident (
                                #(#unpack_fields, )*
                            )
                        }
                    }

                    #vis struct #pack_ident #pack_type_generics ( #( #pack_fields ,)* );

                    impl #pack_impl_generics alkahest::Pack<#ident #type_generics> for #pack_ident #pack_type_generics #pack_where_clause {
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
                    struct #unpacked_ident #unpacked_generics { #( #unpacked_fields ,)* }

                    impl #unpacked_impl_generics alkahest::SchemaUnpack<'alkahest> for #ident #type_generics #where_clause {
                        type Unpacked = #unpacked_ident #unpacked_type_generics;
                    }

                    #[derive(Clone, Copy)]
                    #[repr(C, packed)]
                    struct #packed_ident #generics { #( #packed_fields ,)* }

                    unsafe impl #impl_generics alkahest::Zeroable for #packed_ident #type_generics #where_clause {}
                    unsafe impl #impl_generics alkahest::Pod for #packed_ident #type_generics #where_clause {}

                    impl #impl_generics alkahest::Schema for #ident #type_generics #where_clause {
                        type Packed = #packed_ident #type_generics;

                        fn align() -> usize {
                            #[allow(dead_code)]
                            fn drop_fields(value: #ident) {
                                #( #drop_fields ; )*
                            }

                            1 + (0 #(| #align_masks )*)
                        }

                        fn unpack<'alkahest>(packed: #packed_ident, bytes: &'alkahest [u8]) -> #unpacked_ident #unpacked_type_generics {
                            #unpacked_ident {
                                #(#unpack_fields, )*
                            }
                        }
                    }

                    #vis struct #pack_ident #pack_type_generics { #( #pack_fields ,)* }

                    impl #pack_impl_generics alkahest::Pack<#ident #type_generics> for #pack_ident #pack_type_generics #pack_where_clause {
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
