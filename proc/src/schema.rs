use proc_macro2::TokenStream;
use syn::spanned::Spanned;

pub fn derive(input: proc_macro::TokenStream) -> syn::Result<TokenStream> {
    let input = syn::parse::<syn::DeriveInput>(input)?;
    let ident = &input.ident;
    let vis = &input.vis;

    let mut schema_generics = input.generics.clone();
    let all_field_types: Vec<_>;
    let field_checks: Vec<_>;
    let field_vises: Vec<_>;

    match &input.data {
        syn::Data::Union(data) => {
            return Err(syn::Error::new_spanned(
                data.union_token,
                "Schema cannot be derived for unions",
            ))
        }
        syn::Data::Struct(data) => {
            all_field_types = data.fields.iter().map(|field| &field.ty).collect();
            field_vises = data.fields.iter().map(|field| &field.vis).collect();

            field_checks = match data.fields {
                syn::Fields::Named(_) => data
                    .fields
                    .iter()
                    .enumerate()
                    .map(|(idx, field)| {
                        quote::format_ident!(
                            "__alkahest_check_field_idx_{}_is_{}",
                            field.ident.as_ref().unwrap(),
                            idx
                        )
                    })
                    .collect(),
                _ => Vec::new(),
            };
        }
        syn::Data::Enum(data) => {
            all_field_types = data
                .variants
                .iter()
                .flat_map(|variant| variant.fields.iter().map(|field| &field.ty))
                .collect();

            todo!()
        }
    }

    if !all_field_types.is_empty() {
        let predicates = all_field_types.iter().map(|ty| -> syn::WherePredicate {
            syn::parse_quote_spanned! { ty.span() => #ty: ::alkahest::Schema }
        });

        let where_clause = schema_generics
            .where_clause
            .get_or_insert_with(|| syn::WhereClause {
                where_token: Default::default(),
                predicates: Default::default(),
            });

        where_clause.predicates.extend(predicates);
    }

    match &input.data {
        syn::Data::Union(_) => unreachable!(),
        syn::Data::Struct(_) => {
            let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

            let (schema_impl_generics, schema_type_generics, schema_where_clause) =
                schema_generics.split_for_impl();

            return Ok(quote::quote! {
                impl #schema_impl_generics ::alkahest::Schema for #ident #schema_type_generics #schema_where_clause {}

                impl #impl_generics #ident #type_generics #where_clause {
                    #(
                        #[doc(hidden)]
                        #[inline(always)]
                        #field_vises fn #field_checks() {}
                    )*
                }
            });
        }
        syn::Data::Enum(_) => {
            todo!()
        }
    };
}
