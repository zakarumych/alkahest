use proc_macro2::TokenStream;

use crate::attrs::{parse_attributes, Args, Formula};

struct Config {
    formula: Formula,

    /// Signals if fields should be checked to match on formula.
    /// `false` if `formula` is inferred to `Self`.
    check_fields: bool,

    /// Signals that it can deserialize
    /// formulas with appended fields.
    /// This requires that last field is `SizedFormula`
    non_exhaustive: bool,
}

impl Config {
    fn for_struct(args: Args, data: &syn::DataStruct) -> Self {
        let non_exhaustive = args.non_exhaustive.is_some();
        match args.deserialize.or(args.common) {
            None => {
                // Add predicates that fields implement
                // `SizedFormula + Deserialize<'de, #field_type>`
                // Except that last one if `non_exhaustive` is not set.
                let predicates = data.fields.iter().map(|field| -> syn::WherePredicate {
                        let ty = &field.ty;
                        syn::parse_quote! { #ty: ::alkahest::private::Formula + ::alkahest::private::Deserialize<'de, <#ty as ::alkahest::private::Formula>> }
                    }).collect();

                // Add `'de` generic parameter
                let generics = syn::Generics {
                    lt_token: Some(Default::default()),
                    params: std::iter::once(syn::GenericParam::Lifetime(syn::parse_quote!('de)))
                        .collect(),
                    gt_token: Some(Default::default()),
                    where_clause: Some(syn::WhereClause {
                        where_token: Default::default(),
                        predicates,
                    }),
                };

                Config {
                    formula: Formula {
                        path: syn::parse_quote!(Self),
                        generics,
                    },
                    check_fields: false,
                    non_exhaustive,
                }
            }
            Some(mut formula) => {
                // If no parameters specified, add `'de` parameter
                if formula.generics.params.is_empty() {
                    formula.generics.params.push(syn::parse_quote!('de));
                }
                Config {
                    formula,
                    check_fields: true,
                    non_exhaustive,
                }
            }
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
                formula,
                check_fields,
                non_exhaustive,
            } = Config::for_struct(args, &data);

            let formula_path = &formula.path;

            let mut deserialize_generics = input.generics.clone();

            deserialize_generics.lt_token =
                deserialize_generics.lt_token.or(formula.generics.lt_token);
            deserialize_generics.gt_token =
                deserialize_generics.gt_token.or(formula.generics.gt_token);
            deserialize_generics
                .params
                .extend(formula.generics.params.into_iter());

            if let Some(where_clause) = formula.generics.where_clause {
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
                            "__ALKAHEST_FORMULA_FIELD_{}_IDX",
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

            let field_names = data
                .fields
                .iter()
                .enumerate()
                .map(|(index, field)| match &field.ident {
                    Some(ident) => syn::Member::from(ident.clone()),
                    None => syn::Member::from(index),
                })
                .collect::<Vec<_>>();

            let consume_tail;

            if non_exhaustive {
                consume_tail = quote::quote! {
                    des.read_all_bytes();
                };
            } else {
                consume_tail = quote::quote! {};
            }

            let field_count = data.fields.len();
            let check_field_count = if check_fields && !non_exhaustive {
                quote::quote! {
                    let _: [(); #field_count] = <#formula_path>::__ALKAHEST_FORMULA_FIELD_COUNT;
                }
            } else {
                quote::quote! {}
            };

            let (_impl_generics, type_generics, _where_clause) = input.generics.split_for_impl();
            let (impl_deserialize_generics, _type_deserialize_generics, where_serialize_clause) =
                deserialize_generics.split_for_impl();
            Ok(quote::quote! {
                impl #impl_deserialize_generics ::alkahest::private::Deserialize<'de, #formula_path> for #ident #type_generics #where_serialize_clause {
                    fn deserialize(mut de: ::alkahest::private::Deserializer<'de>) -> ::alkahest::private::Result<Self, ::alkahest::private::Error> {
                        // Checks compilation of code in the block.
                        #[allow(unused)]
                        let _ = || {
                            #(let _: [(); #field_check_idxs] = <#formula_path>::#field_check_names;)*
                        };
                        #check_field_count

                        #(
                            let #field_names = ::alkahest::private::with_formula(|s: &#formula_path| &s.#field_names).read_value(&mut de)?;
                        )*
                        #consume_tail

                        de.finish()?;

                        ::alkahest::private::Result::Ok(#ident {
                            #(#field_names,)*
                        })
                    }

                    fn deserialize_in_place(&mut self, mut de: ::alkahest::private::Deserializer<'de>) -> Result<(), ::alkahest::private::Error> {
                        #(
                            ::alkahest::private::with_formula(|s: &#formula_path| &s.#field_names).read_in_place(&mut self.#field_names, &mut de)?;
                        )*
                        #consume_tail
                        de.finish()
                    }
                }
            })
        }
        syn::Data::Enum(_) => {
            todo!()
        }
    }
}
