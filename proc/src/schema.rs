use proc_macro2::TokenStream;

fn field_param(idx: usize, ident: &Option<syn::Ident>) -> syn::Ident {
    match ident {
        Some(ident) => quote::format_ident!("__{}", ident),
        None => quote::format_ident!("__{}", idx),
    }
}

pub fn derive_schema(input: proc_macro::TokenStream) -> syn::Result<TokenStream> {
    let input = syn::parse::<syn::DeriveInput>(input)?;

    let data = input.data;

    let input = Input {
        vis: input.vis,
        ident: input.ident,
        generics: input.generics,
    };

    match data {
        syn::Data::Struct(data) => derive_schema_struct(input, data),
        syn::Data::Enum(data) => derive_schema_enum(input, data),
        syn::Data::Union(data) => Err(syn::Error::new_spanned(
            data.union_token,
            "Schema cannot be derived for unions",
        )),
    }
}

struct Input {
    vis: syn::Visibility,
    ident: syn::Ident,
    generics: syn::Generics,
}

fn derive_schema_struct(input: Input, data: syn::DataStruct) -> syn::Result<TokenStream> {
    let Input {
        vis,
        ident,
        generics,
    } = input;

    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    let access_ident = quote::format_ident!("{}Access", ident);
    let serialize_ident = quote::format_ident!("{}Serialize", ident);

    let access_generics = syn::Generics {
        lt_token: Some(<syn::Token![<]>::default()),
        params: {
            std::iter::once(syn::parse_quote!('__a))
                .chain(generics.params.iter().cloned())
                .collect()
        },
        gt_token: Some(<syn::Token![>]>::default()),
        where_clause: generics.where_clause.clone(),
    };

    let serialize_generics = syn::Generics {
        lt_token: (!data.fields.is_empty()).then(|| <syn::Token![<]>::default()),
        params: {
            data.fields
                .iter()
                .enumerate()
                .map(|(idx, field)| syn::GenericParam::Type(field_param(idx, &field.ident).into()))
                .collect()
        },
        gt_token: (!data.fields.is_empty()).then(|| <syn::Token![>]>::default()),
        where_clause: None,
    };

    let mut impl_serialize_generics = serialize_generics.clone();
    impl_serialize_generics
        .params
        .extend(generics.params.iter().cloned());
    impl_serialize_generics.where_clause = Some({
        syn::WhereClause {
            where_token: <syn::Token![where]>::default(),
            predicates: {
                data.fields
                    .iter()
                    .enumerate()
                    .map(|(idx, field)| {
                        let ty = &field.ty;
                        syn::WherePredicate::Type(syn::PredicateType {
                            lifetimes: None,
                            bounded_ty: syn::Type::Path(syn::TypePath {
                                qself: None,
                                path: field_param(idx, &field.ident).into(),
                            }),
                            colon_token: <syn::Token![:]>::default(),
                            bounds: std::iter::once::<syn::TypeParamBound>(
                                syn::parse_quote!(::alkahest::Serialize<#ty>),
                            )
                            .collect(),
                        })
                    })
                    .collect()
            },
        }
    });

    let (impl_serialize_impl_generics, impl_serialize_type_generics, impl_serialize_where_clause) =
        impl_serialize_generics.split_for_impl();

    let impl_serialize_header_arguments = match data.fields.is_empty() {
        true => syn::PathArguments::None,
        false => syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
            colon2_token: None,
            lt_token: <syn::Token![<]>::default(),
            args: {
                data.fields
                    .iter()
                    .enumerate()
                    .map(|(idx, field)| {
                        let p = field_param(idx, &field.ident);
                        let ty = &field.ty;
                        syn::GenericArgument::Type(
                            syn::parse_quote!{(<#p as ::alkahest::Serialize<#ty>>::Header, ::alkahest::private::usize)}
                        )
                    })
                    .collect()
            },
            gt_token: <syn::Token![>]>::default(),
        }),
    };

    let fields_ty1 = data.fields.iter().map(|field| &field.ty);
    let fields_ty2 = fields_ty1.clone();
    let fields_ty3 = fields_ty1.clone();
    let fields_ty4 = fields_ty1.clone();
    let fields_ty5 = fields_ty1.clone();
    let fields_ty6 = fields_ty1.clone();
    let fields_ty7 = fields_ty1.clone();
    let fields_ty8 = fields_ty1.clone();
    let fields_ty9 = fields_ty1.clone();
    let fields_ty10 = fields_ty1.clone();

    let fields_p1 = data
        .fields
        .iter()
        .enumerate()
        .map(|(idx, field)| field_param(idx, &field.ident));
    let fields_p2 = fields_p1.clone();
    let fields_p3 = fields_p1.clone();
    let fields_p4 = fields_p1.clone();

    match &data.fields {
        syn::Fields::Named(_) => {
            let fileds_ident1 = data
                .fields
                .iter()
                .map(|field| field.ident.as_ref().unwrap());

            let fileds_ident2 = fileds_ident1.clone();
            let fileds_ident3 = fileds_ident1.clone();
            let fileds_ident4 = fileds_ident1.clone();
            let fileds_ident5 = fileds_ident1.clone();
            let fileds_ident6 = fileds_ident1.clone();
            let fileds_ident7 = fileds_ident1.clone();
            let fileds_ident8 = fileds_ident1.clone();
            let fileds_ident9 = fileds_ident1.clone();
            let fileds_ident10 = fileds_ident1.clone();

            Ok(quote::quote! {
                #vis struct #access_ident #access_generics {
                    #(#fileds_ident1: <#fields_ty1 as ::alkahest::Schema>::Access<'__a>,)*
                }

                impl #impl_generics ::alkahest::Schema for #ident #type_generics #where_clause {
                    type Access<'__a> = #access_ident #access_generics;

                    fn header() -> ::alkahest::private::usize {
                        0 #(+ <#fields_ty2 as ::alkahest::Schema>::header())*
                    }

                    fn has_body() -> ::alkahest::private::bool {
                        false #(|| <#fields_ty3 as ::alkahest::Schema>::has_body())*
                    }

                    fn access(input: &[::alkahest::private::u8]) -> #access_ident<'_> {
                        let mut offset = 0;
                        #access_ident {
                            #(#fileds_ident2: {
                                let cur = offset;
                                offset += <#fields_ty4 as ::alkahest::Schema>::header();
                                <#fields_ty5 as ::alkahest::Schema>::access(&input[cur..])
                            },)*
                        }
                    }
                }

                #[allow(non_camel_case_types)]
                #vis struct #serialize_ident #serialize_generics {
                    #(#fileds_ident3: #fields_p2,)*
                }

                #[allow(non_camel_case_types)]
                impl #impl_serialize_impl_generics ::alkahest::Serialize<#ident> for #serialize_ident #impl_serialize_type_generics #impl_serialize_where_clause {
                    type Header = #serialize_ident #impl_serialize_header_arguments;

                    #[inline]
                    fn serialize_header(header: Self::Header, output: &mut [::alkahest::private::u8], offset: ::alkahest::private::usize) -> ::alkahest::private::bool {
                        let header_size = <#ident as ::alkahest::Schema>::header();

                        if output.len() < header_size {
                            return false;
                        }

                        let mut total_offset = offset;
                        let mut output = output;
                        #(
                            let (field_header, field_offset) = header.#fileds_ident4;
                            let header_size = <#fields_ty6 as ::alkahest::Schema>::header();

                            let (head, tail) = output.split_at_mut(header_size);
                            output = tail;

                            <#fields_p1 as ::alkahest::Serialize<#fields_ty7>>::serialize_header(field_header, head, total_offset + field_offset);
                            total_offset -= header_size;
                        )*

                        let _ = (output, total_offset);
                        true
                    }

                    #[inline]
                    fn serialize_body(self, output: &mut [::alkahest::private::u8]) -> ::alkahest::private::Result<(Self::Header, ::alkahest::private::usize), ::alkahest::private::usize> {
                        let mut headers_opt = #serialize_ident {
                            #(#fileds_ident5: None,)*
                        };

                        let mut written = 0;
                        let mut exhausted = false;
                        #(
                            let offset = written;
                            if !exhausted {
                                match <#fields_p3 as ::alkahest::Serialize<#fields_ty9>>::serialize_body(self.#fileds_ident6, &mut output[offset..]) {
                                    Ok((header, size)) => {
                                        headers_opt.#fileds_ident7 = Some((header, offset));
                                        written += size;
                                    }
                                    Err(size) => {
                                        exhausted = true;
                                        written += size;
                                    }
                                }
                            } else {
                                let size = <#fields_p4 as ::alkahest::Serialize<#fields_ty10>>::body_size(self.#fileds_ident8);
                                written += size;
                            }
                        )*

                        if exhausted {
                            Err(written)
                        } else {
                            let header = #serialize_ident {
                                #(#fileds_ident9: headers_opt.#fileds_ident10.unwrap(),)*
                            };
                            Ok((header, written))
                        }
                    }
                }
            })
        }
        syn::Fields::Unnamed(_) => {
            let fileds_idx1 =
                (0..data.fields.len()).map(|idx| syn::Member::Unnamed(syn::Index::from(idx)));
            let fileds_idx2 = fileds_idx1.clone();
            let fileds_idx3 = fileds_idx1.clone();
            let fileds_idx4 = fileds_idx1.clone();
            let fileds_idx5 = fileds_idx1.clone();

            let field_nones = (0..data.fields.len()).map(|idx| quote::format_ident!("None"));

            Ok(quote::quote! {
                #vis struct #access_ident #access_generics(
                    #(<#fields_ty1 as ::alkahest::Schema>::Access<'__a>,)*
                );

                impl #impl_generics ::alkahest::Schema for #ident #type_generics #where_clause {
                    type Access<'__a> = #access_ident #access_generics;

                    fn header() -> ::alkahest::private::usize {
                        0 #(+ <#fields_ty2 as ::alkahest::Schema>::header())*
                    }

                    fn has_body() -> ::alkahest::private::bool {
                        false #(|| <#fields_ty3 as ::alkahest::Schema>::has_body())*
                    }

                    fn access(input: &[::alkahest::private::u8]) -> #access_ident<'_> {
                        let mut offset = 0;
                        #access_ident(
                            #({
                                let cur = offset;
                                offset += <#fields_ty4 as ::alkahest::Schema>::header();
                                <#fields_ty5 as ::alkahest::Schema>::access(&input[cur..])
                            },)*
                        )
                    }
                }

                #[allow(non_camel_case_types)]
                #vis struct #serialize_ident #serialize_generics(
                    #(#fields_p2,)*
                );

                #[allow(non_camel_case_types)]
                impl #impl_serialize_impl_generics ::alkahest::Serialize<#ident> for #serialize_ident #impl_serialize_type_generics #impl_serialize_where_clause {
                    type Header = #serialize_ident #impl_serialize_header_arguments;

                    #[inline]
                    fn serialize_header(header: Self::Header, output: &mut [::alkahest::private::u8], offset: ::alkahest::private::usize) -> ::alkahest::private::bool {
                        let header_size = <#ident as ::alkahest::Schema>::header();

                        if output.len() < header_size {
                            return false;
                        }

                        let mut total_offset = offset;
                        let mut output = output;
                        #(
                            let (field_header, field_offset) = header.#fileds_idx1;
                            let header_size = <#fields_ty6 as ::alkahest::Schema>::header();

                            let (head, tail) = output.split_at_mut(header_size);
                            output = tail;

                            <#fields_p1 as ::alkahest::Serialize<#fields_ty7>>::serialize_header(field_header, head, total_offset + field_offset);
                            total_offset -= header_size;
                        )*

                        let _ = (output, total_offset);
                        true
                    }

                    #[inline]
                    fn serialize_body(self, output: &mut [::alkahest::private::u8]) -> ::alkahest::private::Result<(Self::Header, ::alkahest::private::usize), ::alkahest::private::usize> {
                        let mut headers_opt = #serialize_ident(
                            #(#field_nones,)*
                        );

                        let mut written = 0;
                        let mut exhausted = false;
                        #(
                            let offset = written;
                            if !exhausted {
                                match <#fields_p3 as ::alkahest::Serialize<#fields_ty9>>::serialize_body(self.#fileds_idx2, &mut output[offset..]) {
                                    Ok((header, size)) => {
                                        headers_opt.#fileds_idx3 = Some((header, offset));
                                        written += size;
                                    }
                                    Err(size) => {
                                        exhausted = true;
                                        written += size;
                                    }
                                }
                            } else {
                                let size = <#fields_p4 as ::alkahest::Serialize<#fields_ty10>>::body_size(self.#fileds_idx4);
                                written += size;
                            }
                        )*

                        if exhausted {
                            Err(written)
                        } else {
                            let header = #serialize_ident(
                                #(headers_opt.#fileds_idx5.unwrap(),)*
                            );
                            Ok((header, written))
                        }
                    }
                }
            })
        }
        syn::Fields::Unit => todo!(),
    }
}

fn derive_schema_enum(input: Input, data: syn::DataEnum) -> syn::Result<TokenStream> {
    let _ = input;
    return Err(syn::Error::new_spanned(
        data.enum_token,
        "Schema cannot be derived for enums just yet",
    ));
}
