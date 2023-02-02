use proc_macro2::TokenStream;
use syn::spanned::Spanned;

use crate::attrs::parse_attributes;

pub fn derive(input: proc_macro::TokenStream, sized: bool) -> syn::Result<TokenStream> {
    let input = syn::parse::<syn::DeriveInput>(input)?;
    let ident = &input.ident;

    let args = parse_attributes(&input.attrs)?;
    if sized {
        if let Some(non_exhaustive) = args.non_exhaustive {
            return Err(syn::Error::new_spanned(
                non_exhaustive,
                "SizedSchema cannot be non-exhaustive",
            ));
        }
    }

    if let Some(schema) = args
        .serialize
        .or(args.deserialize)
        .or(args.common)
        .or(args.no_reference.flatten())
    {
        return Err(syn::Error::new_spanned(
            schema.ty,
            "Schema type should not be specified for `Serialize` and `Deserialize` when type is also `Schema`",
        ));
    }

    if args.variant.is_some() {
        return Err(syn::Error::new_spanned(
            input,
            "Variant should not be specified for `Serialize` when type is also `Schema`",
        ));
    }

    let mut schema_generics = input.generics.clone();
    let all_field_types: Vec<_>;
    let field_check_names: Vec<_>;
    let field_check_ids: Vec<_>;
    let field_count;

    match &input.data {
        syn::Data::Union(data) => {
            return Err(syn::Error::new_spanned(
                data.union_token,
                "Schema cannot be derived for unions",
            ))
        }
        syn::Data::Struct(data) => {
            all_field_types = data.fields.iter().map(|field| &field.ty).collect();
            field_count = data.fields.len();

            field_check_names = match data.fields {
                syn::Fields::Named(_) => data
                    .fields
                    .iter()
                    .map(|field| {
                        quote::format_ident!(
                            "__alkahest_schema_field_{}_idx_is",
                            field.ident.as_ref().unwrap(),
                        )
                    })
                    .collect(),
                _ => Vec::new(),
            };

            field_check_ids = match data.fields {
                syn::Fields::Named(_) => (0..data.fields.len()).collect(),
                _ => Vec::new(),
            };
        }
        syn::Data::Enum(_) => {
            todo!()
        }
    }

    if !all_field_types.is_empty() {
        let predicates = all_field_types.iter().map(|ty| -> syn::WherePredicate {
            syn::parse_quote_spanned! { ty.span() => #ty: ::alkahest::Schema }
        });

        let where_clause = schema_generics.make_where_clause();
        where_clause.predicates.extend(predicates);
    }

    match &input.data {
        syn::Data::Union(_) => unreachable!(),
        syn::Data::Struct(_) => {
            let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

            let (schema_impl_generics, schema_type_generics, schema_where_clause) =
                schema_generics.split_for_impl();

            let mut tokens = quote::quote! {
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
                    pub const fn __alkahest_schema_field_count() -> [(); #field_count] {
                        [(); #field_count]
                    }
                }

                impl #schema_impl_generics ::alkahest::Schema for #ident #schema_type_generics #schema_where_clause {}
            };

            if sized {
                let mut sized_schema_generics = schema_generics.clone();

                if !all_field_types.is_empty() {
                    let predicates = all_field_types.iter().map(|ty| -> syn::WherePredicate {
                        syn::parse_quote_spanned! { ty.span() => #ty: ::alkahest::SizedSchema }
                    });

                    let where_clause = sized_schema_generics.make_where_clause();
                    where_clause.predicates.extend(predicates);
                }

                let (
                    sized_schema_impl_generics,
                    sized_schema_type_generics,
                    sized_schema_where_clause,
                ) = sized_schema_generics.split_for_impl();

                tokens.extend(quote::quote! {
                    impl #sized_schema_impl_generics ::alkahest::SizedSchema for #ident #sized_schema_type_generics #sized_schema_where_clause {
                        const SIZE: ::alkahest::private::usize = 0 #( + <#all_field_types as ::alkahest::SizedSchema>::SIZE)*;
                    }
                });
            }

            Ok(tokens)
        }
        syn::Data::Enum(_) => {
            todo!()
        }
    }
}
