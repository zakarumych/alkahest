use proc_easy::{EasyAttributes, EasyMaybe};
use proc_macro2::TokenStream;
use syn::spanned::Spanned;

proc_easy::easy_token!(schema);

proc_easy::easy_parse! {
    struct VariantRef {
        token: syn::Token![=>],
        ident: syn::Ident,
    }
}

proc_easy::easy_parse! {
    struct SchemaRef {
        ty: syn::Type,
        variant: EasyMaybe<VariantRef>,
    }
}

proc_easy::easy_argument! {
    struct SchemaArg {
        token: schema,
        schema: proc_easy::EasyParenthesized<SchemaRef>,
    }
}

proc_easy::easy_attributes! {
    @(alkahest)
    struct SerializeAttributes {
        schema: Option<SchemaArg>,
    }
}

pub fn derive(input: proc_macro::TokenStream) -> syn::Result<TokenStream> {
    let input = syn::parse::<syn::DeriveInput>(input)?;
    let SerializeAttributes { schema } = EasyAttributes::parse(&input.attrs, input.span())?;

    let (schema, variant): (syn::Type, Option<syn::Ident>) = match schema {
        Some(arg) => {
            let variant = match arg.schema.0.variant {
                EasyMaybe::Just(variant) => Some(variant.ident),
                EasyMaybe::Nothing => None,
            };
            (arg.schema.0.ty, variant)
        }
        None => (syn::parse_quote!(Self), None),
    };

    let ident = &input.ident;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

    match input.data {
        syn::Data::Union(_) => Err(syn::Error::new_spanned(
            input,
            "Serialize cannot be derived for unions",
        )),
        syn::Data::Struct(data) => {
            let field_checks = match data.fields {
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

            let field_names = data
                .fields
                .iter()
                .enumerate()
                .map(|(index, field)| match &field.ident {
                    Some(ident) => syn::Member::from(ident.clone()),
                    None => syn::Member::from(index),
                })
                .collect::<Vec<_>>();

            Ok(quote::quote! {
                impl #impl_generics ::alkahest::Serialize<#schema> for #ident #type_generics #where_clause {
                    fn serialize(self, offset: usize, output: &mut [u8]) -> Result<(usize, usize), usize> {
                        #(
                            <#schema>::#field_checks();
                        )*

                        let mut ser = ::alkahest::Serializer::new(offset, output);

                        #(
                            ::alkahest::private::with_schema(|s: &#schema| &s.#field_names).serialize_value(&mut ser, self.#field_names)?;
                        )*

                        Ok(ser.finish())
                    }
                }
            })
        }
        syn::Data::Enum(data) => {
            todo!()
        }
    }
}
