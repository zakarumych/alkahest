use proc_macro2::TokenStream;

use crate::attrs::{parse_attributes, Args};

struct Config {
    schema_type: syn::Type,

    /// Additional generics required to implement `Serialize<#schema_type_type>`
    /// Specified in attributes or inferred when schema is `Self`.
    schema_generics: syn::Generics,

    /// Signals if fields should be checked to match on schema.
    /// `false` if `schema` is inferred to `Self`.
    check_fields: bool,

    /// Signals that it can deserialize
    /// schemas with appended fields.
    /// This requires that last field is `SizedSchema`
    non_exhaustive: bool,
}

impl Config {
    fn for_struct(args: Args, data: &syn::DataStruct) -> Self {
        let non_exhaustive = args.non_exhaustive.is_some();
        match args.deserialize.or(args.common) {
            None => {
                let schema_generics = if data.fields.is_empty() {
                    Default::default()
                } else {
                    // Add predicates that fields implement
                    // `SizedSchema + Serialize<#field_type>`
                    // Except that last one if `non_exhaustive` is not set.

                    let count = data.fields.len();
                    let predicates = data.fields.iter().enumerate().map(|(idx, field)| -> syn::WherePredicate {
                        let ty = &field.ty;

                        if non_exhaustive || idx + 1 < count {
                            syn::parse_quote! { #ty: ::alkahest::SizedSchema + ::alkahest::Deserialize<'de, #ty> }
                        } else {
                            debug_assert_eq!(idx + 1, count);
                            syn::parse_quote! { #ty: ::alkahest::Schema + ::alkahest::Deserialize<'de, #ty> }
                        }
                    }).collect();

                    syn::Generics {
                        lt_token: None,
                        params: Default::default(),
                        gt_token: None,
                        where_clause: Some(syn::WhereClause {
                            where_token: Default::default(),
                            predicates,
                        }),
                    }
                };

                Config {
                    schema_type: syn::parse_quote!(Self),
                    schema_generics,
                    check_fields: false,
                    non_exhaustive,
                }
            }
            Some(schema) => Config {
                schema_type: schema.ty,
                schema_generics: schema.generics,
                check_fields: true,
                non_exhaustive,
            },
        }
    }
}

pub fn derive(input: proc_macro::TokenStream) -> syn::Result<TokenStream> {
    let input = syn::parse::<syn::DeriveInput>(input)?;
    let args = parse_attributes(&input.attrs)?;

    let ident = &input.ident;

    match input.data {
        syn::Data::Union(_) => Err(syn::Error::new_spanned(
            input,
            "Serialize cannot be derived for unions",
        )),
        syn::Data::Struct(data) => {
            let Config {
                schema_type,
                schema_generics,
                check_fields,
                non_exhaustive,
            } = Config::for_struct(args, &data);

            let mut deserialize_generics = input.generics.clone();

            deserialize_generics.lt_token =
                deserialize_generics.lt_token.or(schema_generics.lt_token);
            deserialize_generics.gt_token =
                deserialize_generics.gt_token.or(schema_generics.gt_token);
            deserialize_generics
                .params
                .extend(schema_generics.params.into_iter());
            deserialize_generics
                .params
                .push(syn::GenericParam::Lifetime(syn::LifetimeDef {
                    attrs: Vec::new(),
                    lifetime: syn::Lifetime::new("'de", proc_macro2::Span::call_site()),
                    colon_token: None,
                    bounds: Default::default(),
                }));

            if let Some(where_clause) = schema_generics.where_clause {
                deserialize_generics
                    .make_where_clause()
                    .predicates
                    .extend(where_clause.predicates);
            }

            let field_check_names = match (check_fields, &data.fields) {
                (true, syn::Fields::Named(_)) => data
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

            let field_check_idxs = match (check_fields, &data.fields) {
                (true, syn::Fields::Named(_)) => (0..data.fields.len()).collect(),
                _ => Vec::new(),
            };

            let mut field_names = data
                .fields
                .iter()
                .enumerate()
                .map(|(index, field)| match &field.ident {
                    Some(ident) => syn::Member::from(ident.clone()),
                    None => syn::Member::from(index),
                })
                .collect::<Vec<_>>();

            let mut last_field_name = vec![];
            let field_names_no_last;
            let consume_tail;

            if non_exhaustive {
                field_names_no_last = field_names;
                consume_tail = vec![quote::quote! {
                    des.consume_tail();
                }];
            } else {
                if let Some(last) = field_names.pop() {
                    last_field_name.push(last);
                }
                field_names_no_last = field_names;
                consume_tail = vec![];
            }

            let field_count = data.fields.len();
            let check_field_count = if check_fields {
                quote::quote! {
                    let _: [(); #field_count] = <#schema_type>::__alkahest_schema_field_count();
                }
            } else {
                quote::quote! {}
            };

            let (_impl_generics, type_generics, _where_clause) = input.generics.split_for_impl();
            let (impl_deserialize_generics, _type_deserialize_generics, where_serialize_clause) =
                deserialize_generics.split_for_impl();
            Ok(quote::quote! {
                impl #impl_deserialize_generics ::alkahest::Deserialize<'de, #schema_type> for #ident #type_generics #where_serialize_clause {
                    fn deserialize(len: ::alkahest::private::usize, input: &'de [::alkahest::private::u8]) -> ::alkahest::private::Result<Self, ::alkahest::DeserializeError> {
                        // Checks compilation of code in the block.
                        #[allow(unused)]
                        let _ = || {
                            #(let _: [(); #field_check_idxs] = <#schema_type>::#field_check_names();)*
                        };
                        #check_field_count

                        let mut des = ::alkahest::Deserializer::new(len, input);

                        #(
                            let #field_names_no_last = ::alkahest::private::with_schema(|s: &#schema_type| &s.#field_names_no_last).deserialize_sized(&mut des)?;
                        )*
                        #(
                            let #last_field_name = ::alkahest::private::with_schema(|s: &#schema_type| &s.#last_field_name).deserialize_rest(&mut des)?;
                        )*
                        #(
                            #consume_tail
                        )*

                        des.finish_checked()?;

                        ::alkahest::private::Result::Ok(#ident {
                            #(#field_names_no_last,)*
                            #(#last_field_name,)*
                        })
                    }

                    fn deserialize_in_place(&mut self, len: usize, input: &[u8]) -> Result<(), ::alkahest::DeserializeError> {
                        todo!()
                    }
                }
            })
        }
        syn::Data::Enum(_) => {
            todo!()
        }
    }
}
