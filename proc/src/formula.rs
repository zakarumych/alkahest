use proc_macro2::{Delimiter, Group, Ident, Span, TokenStream};
use quote::{ToTokens, TokenStreamExt};
use syn::punctuated::Punctuated;

use crate::args::{FormulaArgs, FormulaDeriveArgs};

struct SizeFields<'a>(&'a syn::Generics);

impl ToTokens for SizeFields<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if self.0.type_params().next().is_none() {
            return;
        }

        tokens.append(Group::new(Delimiter::Parenthesis, {
            let mut tokens = TokenStream::new();

            tokens.append(proc_macro2::Punct::new(':', proc_macro2::Spacing::Joint));
            tokens.append(proc_macro2::Punct::new(':', proc_macro2::Spacing::Alone));
            tokens.append(Ident::new("alkahest", Span::call_site()));
            tokens.append(proc_macro2::Punct::new(':', proc_macro2::Spacing::Joint));
            tokens.append(proc_macro2::Punct::new(':', proc_macro2::Spacing::Alone));
            tokens.append(Ident::new("private", Span::call_site()));
            tokens.append(proc_macro2::Punct::new(':', proc_macro2::Spacing::Joint));
            tokens.append(proc_macro2::Punct::new(':', proc_macro2::Spacing::Alone));
            tokens.append(Ident::new("PhantomData", Span::call_site()));

            self.0.lt_token.to_tokens(&mut tokens);

            tokens.append(Group::new(Delimiter::Parenthesis, {
                let mut tokens = TokenStream::new();
                for param in self.0.params.pairs() {
                    if let syn::GenericParam::Type(ty_param) = param.value() {
                        ty_param.ident.to_tokens(&mut tokens);
                        param.punct().to_tokens(&mut tokens);
                    }
                }
                tokens
            }));

            self.0.gt_token.to_tokens(&mut tokens);

            tokens
        }));
    }
}

fn make_formula_generics(
    generics: &syn::Generics,
    generate_type_definition: bool,
) -> syn::Generics {
    let mut result = generics.clone();
    if generate_type_definition {
        for param in generics.type_params() {
            let ident = &param.ident;
            result
                .make_where_clause()
                .predicates
                .push(syn::parse_quote!(#ident: ::alkahest::Element));
        }
    }
    result
}

pub(crate) fn needs_field_bound(
    ty: &syn::Type,
    generics: &syn::Generics,
    self_name: &Ident,
) -> bool {
    fn contains(tokens: TokenStream, name: &Ident) -> bool {
        tokens.into_iter().any(|token| match token {
            proc_macro2::TokenTree::Ident(ident) => ident == *name,
            proc_macro2::TokenTree::Group(group) => contains(group.stream(), name),
            _ => false,
        })
    }
    let tokens = ty.to_token_stream();
    !contains(tokens.clone(), self_name)
        && !contains(tokens.clone(), &Ident::new("Self", Span::call_site()))
        && generics
            .type_params()
            .any(|param| contains(tokens.clone(), &param.ident))
}

pub(crate) fn field_helper_name(
    version: u32,
    variant: Option<&Ident>,
    idx: usize,
    name: Option<&Ident>,
) -> Ident {
    let field = name
        .map(ToString::to_string)
        .unwrap_or_else(|| idx.to_string());
    match variant {
        Some(variant) => {
            quote::format_ident!("__alkahest_field_v{}_{}_{}", version, variant, field)
        }
        None => quote::format_ident!("__alkahest_field_v{}_{}", version, field),
    }
}

pub(crate) fn validate_annotations(data: &syn::Data) -> syn::Result<()> {
    let fields: Vec<_> = match data {
        syn::Data::Struct(data) => data.fields.iter().collect(),
        syn::Data::Enum(data) => {
            for variant in &data.variants {
                for attr in &variant.attrs {
                    if attr.path().is_ident("alkahest") {
                        return Err(syn::Error::new_spanned(
                            attr,
                            "version belongs on a formula or field, not a variant",
                        ));
                    }
                }
            }
            data.variants.iter().flat_map(|v| v.fields.iter()).collect()
        }
        syn::Data::Union(_) => Vec::new(),
    };
    for field in fields {
        crate::args::field_version(&field.attrs)?;
    }
    Ok(())
}

pub(crate) fn reject_field_annotations(data: &syn::Data) -> syn::Result<()> {
    let fields: Vec<_> = match data {
        syn::Data::Struct(data) => data.fields.iter().collect(),
        syn::Data::Enum(data) => data.variants.iter().flat_map(|v| v.fields.iter()).collect(),
        syn::Data::Union(_) => Vec::new(),
    };
    for field in fields {
        if let Some(attr) = field
            .attrs
            .iter()
            .find(|attr| attr.path().is_ident("alkahest"))
        {
            return Err(syn::Error::new_spanned(
                attr,
                "child versions must be declared on the formula fields",
            ));
        }
    }
    Ok(())
}

fn field_metadata(
    version: u32,
    variant: Option<&Ident>,
    fields: impl Iterator<Item = (Option<Ident>, syn::Type, u32)>,
) -> (TokenStream, TokenStream) {
    let (helpers, metadata): (Vec<_>, Vec<_>) = fields.enumerate().map(|(idx,(name, ty, child))| {
        let helper = field_helper_name(version, variant, idx, name.as_ref());
        let label = name.as_ref().map(ToString::to_string).unwrap_or_else(|| idx.to_string());
        let helpers = quote::quote! {
            #[doc(hidden)]
            #[allow(non_snake_case)]
            pub fn #helper() -> ::alkahest::private::WithElement<#ty, #child> {
                ::alkahest::private::with_element_versioned::<(), #ty, #child>(|_| unreachable!())
            }
        };
        let metadata = quote::quote! {
            ::alkahest::Field { name: #label, element: ::alkahest::element_schema::<#ty, #child>() }
        };
        (helpers, metadata)
    }).unzip();
    (
        quote::quote!(#(#helpers)*),
        quote::quote!(&[#(#metadata),*]),
    )
}

fn make_size_generics(formula_generics: &syn::Generics) -> syn::Generics {
    let mut size_generics = formula_generics.clone();

    size_generics.lt_token.get_or_insert_default();
    size_generics.gt_token.get_or_insert_default();

    size_generics
        .params
        .push(syn::GenericParam::Const(syn::ConstParam {
            attrs: Vec::new(),
            const_token: syn::Token![const](Span::call_site()),
            ident: syn::Ident::new("__SIZE_BYTES", Span::call_site()),
            colon_token: syn::Token![:](Span::call_site()),
            ty: syn::Type::Path(syn::TypePath {
                attrs: Vec::new(),
                qself: None,
                path: syn::Path {
                    leading_colon: None,
                    segments: std::iter::once(syn::PathSegment {
                        ident: syn::Ident::new("usize", Span::call_site()),
                        arguments: syn::PathArguments::None,
                    })
                    .collect(),
                },
            }),
            default: None,
        }));

    size_generics
}

enum VariantInhabited {
    True,
    Expr(proc_macro2::TokenStream),
}

pub fn derive_unit(
    ident: syn::Ident,
    generics: syn::Generics,
    version: u32,
    linked_versions: &[TokenStream],
    generate_type_definition: bool,
    tokens: &mut TokenStream,
) {
    let formula_generics = make_formula_generics(&generics, generate_type_definition);

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let (formula_impl_generics, formula_ty_generics, formula_where_clause) =
        formula_generics.split_for_impl();

    let size_fields = SizeFields(&generics);

    if generate_type_definition {
        tokens.extend(quote::quote! {
            pub struct #ident #generics #size_fields;
        });
    }

    tokens.extend(quote::quote! {
        #[doc(hidden)]
        impl #impl_generics #ident #ty_generics #where_clause {
            const __ALKAHEST_FIELD_COUNT: usize = 0;

            #[allow(unused)]
            fn __alkahest_construct() -> Self {
                #ident
            }
        }

        impl #formula_impl_generics ::alkahest::Formula<#version> for #ident #formula_ty_generics #formula_where_clause {
            type StackSize<const __SIZE_BYTES: usize> = ::alkahest::ExactSize<0>;
            type HeapSize<const __SIZE_BYTES: usize> = ::alkahest::ExactSize<0>;

            const INHABITED: bool = true;
            const VERSIONS: &'static [fn() -> &'static ::alkahest::Schema] = &[::alkahest::schema::<Self, #version>, #(#linked_versions),*];
            const SCHEMA: ::alkahest::Schema = ::alkahest::Schema::new::<Self, #version>(::alkahest::SchemaKind::Record(&[]));
        }
    });
}

#[allow(clippy::too_many_arguments)]
pub fn derive_tuple(
    ident: syn::Ident,
    generics: syn::Generics,
    fields: impl Iterator<Item = syn::Type>,
    version: u32,
    linked_versions: &[TokenStream],
    field_versions: &[u32],
    generate_type_definition: bool,
    tokens: &mut TokenStream,
) {
    let stack_size_ident = quote::format_ident!("__Alkahest_{}StackSize", ident);
    let heap_size_ident = quote::format_ident!("__Alkahest_{}HeapSize", ident);

    let formula_generics = make_formula_generics(&generics, generate_type_definition);
    let size_generics = make_size_generics(&formula_generics);

    let (formula_impl_generics, formula_ty_generics, formula_where_clause) =
        formula_generics.split_for_impl();

    let (size_impl_generics, size_ty_generics, size_where_clause) = size_generics.split_for_impl();

    let size_fields = SizeFields(&generics);

    // Extract information about each field
    let fields = fields.enumerate().map(|(idx, element)| {
        let child = field_versions.get(idx).copied().unwrap_or(0);
        let stack_size = quote::quote! {
            ::alkahest::stack_size_versioned::<#element, __SIZE_BYTES, #child>()
        };

        let heap_size = quote::quote! {
            ::alkahest::heap_size_versioned::<#element, __SIZE_BYTES, #child>()
        };

        let inhabited = quote::quote! {
            ::alkahest::inhabited_versioned::<#element, #child>()
        };

        (element, stack_size, heap_size, inhabited)
    });

    let (field_tys, field_stack_sizes, field_heap_sizes, field_inhabiteds): (
        Vec<_>,
        Vec<_>,
        Vec<_>,
        Vec<_>,
    ) = fields.collect();

    let mut field_stack_sizes = field_stack_sizes.iter();
    let mut field_heap_sizes = field_heap_sizes.iter();
    let mut field_inhabiteds = field_inhabiteds.iter();

    // Generate stack size expression.
    let stack_size = match field_stack_sizes.len() {
        0 => quote::quote! {::alkahest::SizeBound::Exact(0)},
        1 => {
            let first = field_stack_sizes.next().unwrap();
            quote::quote! { #first }
        }
        _ => {
            let first = field_stack_sizes.next().unwrap();
            quote::quote! {{
                let mut total = #first;

                #(
                    let next = #field_stack_sizes;
                    if total.is_unbounded() && !next.is_zero() {
                        panic!("Composite formula contains stack-unbounded element that is not the last one");
                    }
                    total = total.padded().add(next);
                )*

                total
            }}
        }
    };

    // Generate heap size expression.
    let heap_size = match field_heap_sizes.next() {
        None => quote::quote! {::alkahest::SizeBound::Exact(0)},
        Some(first) => {
            quote::quote! { #first #( .add(#field_heap_sizes) )* }
        }
    };

    // Generate inhabited expression.
    let inhabited = match field_inhabiteds.next() {
        None => quote::quote! { true },
        Some(first) => {
            quote::quote! { #first #( && #field_inhabiteds )* }
        }
    };

    let fields_count = field_tys.len();

    if generate_type_definition {
        tokens.extend(quote::quote! {
        pub struct #ident #generics ( #(pub #field_tys,)* );
        });
    }

    let bound_fields = (0..fields_count)
        .map(|idx| quote::format_ident!("__alkahest__field_{}", idx))
        .collect::<Vec<_>>();

    let (field_helpers, schema_fields) = field_metadata(
        version,
        None,
        field_tys
            .iter()
            .cloned()
            .enumerate()
            .map(|(idx, ty)| (None, ty, field_versions.get(idx).copied().unwrap_or(0))),
    );

    tokens.extend(quote::quote! {
        #[allow(non_camel_case_types)]
        pub struct #stack_size_ident #size_generics #size_fields;

        impl #size_impl_generics ::alkahest::SizeType for #stack_size_ident #size_ty_generics #size_where_clause {
            const VALUE: ::alkahest::SizeBound = #stack_size;
        }

        #[allow(non_camel_case_types)]
        pub struct #heap_size_ident #size_generics #size_fields;

        impl #size_impl_generics ::alkahest::SizeType for #heap_size_ident #size_ty_generics #size_where_clause {
            const VALUE: ::alkahest::SizeBound = #heap_size;
        }

        impl #formula_impl_generics #ident #formula_ty_generics #formula_where_clause {
            #field_helpers
            const __ALKAHEST_FIELD_COUNT: usize = #fields_count;

            #[allow(unused)]
            fn __alkahest_construct( #(#bound_fields: #field_tys),* ) -> Self {
                #ident ( #(#bound_fields,)* )
            }
        }

        impl #formula_impl_generics ::alkahest::Formula<#version> for #ident #formula_ty_generics #formula_where_clause {
            type StackSize<const __SIZE_BYTES: usize> = #stack_size_ident #size_ty_generics;
            type HeapSize<const __SIZE_BYTES: usize> = #heap_size_ident #size_ty_generics;

            const INHABITED: bool = #inhabited;
            const VERSIONS: &'static [fn() -> &'static ::alkahest::Schema] = &[::alkahest::schema::<Self, #version>, #(#linked_versions),*];
            const SCHEMA: ::alkahest::Schema = ::alkahest::Schema::new::<Self, #version>(::alkahest::SchemaKind::Record(#schema_fields));
        }
    });
}

pub fn derive_record(
    ident: syn::Ident,
    generics: syn::Generics,
    fields: impl Iterator<Item = (Ident, syn::Type)>,
    version: u32,
    linked_versions: &[TokenStream],
    field_versions: &[u32],
    generate_type_definition: bool,
    tokens: &mut TokenStream,
) {
    let stack_size_ident = quote::format_ident!("__Alkahest_{}StackSize", ident);
    let heap_size_ident = quote::format_ident!("__Alkahest_{}HeapSize", ident);

    let formula_generics = make_formula_generics(&generics, generate_type_definition);
    let size_generics = make_size_generics(&formula_generics);

    let (formula_impl_generics, formula_ty_generics, formula_where_clause) =
        formula_generics.split_for_impl();

    let (size_impl_generics, size_ty_generics, size_where_clause) = size_generics.split_for_impl();

    let size_fields = SizeFields(&generics);

    // Extract information about each field
    let fields = fields.enumerate().map(|(idx, (name, element))| {
        let child = field_versions.get(idx).copied().unwrap_or(0);
        let stack_size = quote::quote! {
            ::alkahest::stack_size_versioned::<#element, __SIZE_BYTES, #child>()
        };

        let heap_size = quote::quote! {
            ::alkahest::heap_size_versioned::<#element, __SIZE_BYTES, #child>()
        };

        let inhabited = quote::quote! {
            ::alkahest::inhabited_versioned::<#element, #child>()
        };

        (name, element, stack_size, heap_size, inhabited)
    });

    let (field_names, field_tys, field_stack_sizes, field_heap_sizes, field_inhabiteds): (
        Vec<_>,
        Vec<_>,
        Vec<_>,
        Vec<_>,
        Vec<_>,
    ) = fields.collect();

    let mut field_stack_sizes = field_stack_sizes.iter();
    let mut field_heap_sizes = field_heap_sizes.iter();
    let mut field_inhabiteds = field_inhabiteds.iter();

    // Generate stack size expression.
    let stack_size = match field_stack_sizes.len() {
        0 => quote::quote! {::alkahest::SizeBound::Exact(0)},
        1 => {
            let first = field_stack_sizes.next().unwrap();
            quote::quote! { #first }
        }
        _ => {
            let first = field_stack_sizes.next().unwrap();
            quote::quote! {{
                let mut total = #first;

                #(
                    let next = #field_stack_sizes;
                    if total.is_unbounded() && !next.is_zero() {
                        panic!("Composite formula contains stack-unbounded element that is not the last one");
                    }
                    total = total.padded().add(next);
                )*

                total
            }}
        }
    };

    // Generate heap size expression.
    let heap_size = match field_heap_sizes.next() {
        None => quote::quote! {::alkahest::SizeBound::Exact(0)},
        Some(first) => {
            quote::quote! { #first #( .add(#field_heap_sizes) )* }
        }
    };

    // Generate inhabited expression.
    let inhabited = match field_inhabiteds.next() {
        None => quote::quote! { true },
        Some(first) => {
            quote::quote! { #first #( && #field_inhabiteds )* }
        }
    };

    let field_order_names = field_names
        .iter()
        .map(|name| quote::format_ident!("__ALKAHEST_ORDER_OF_{}", name));

    let fields_order =
        (0..field_names.len()).map(|order| proc_macro2::Literal::usize_unsuffixed(order));

    let fields_count = field_names.len();

    if generate_type_definition {
        tokens.extend(quote::quote! {
            pub struct #ident #generics { #(pub #field_names: #field_tys,)* }
        });
    }

    let (field_helpers, schema_fields) = field_metadata(
        version,
        None,
        field_tys.iter().cloned().enumerate().map(|(idx, ty)| {
            (
                Some(field_names[idx].clone()),
                ty,
                field_versions.get(idx).copied().unwrap_or(0),
            )
        }),
    );

    tokens.extend(quote::quote! {
        #[allow(non_camel_case_types)]
        pub struct #stack_size_ident #size_generics #size_fields;

        impl #size_impl_generics ::alkahest::SizeType for #stack_size_ident #size_ty_generics #size_where_clause {
            const VALUE: ::alkahest::SizeBound = #stack_size;
        }

        #[allow(non_camel_case_types)]
        pub struct #heap_size_ident #size_generics #size_fields;

        impl #size_impl_generics ::alkahest::SizeType for #heap_size_ident #size_ty_generics #size_where_clause {
            const VALUE: ::alkahest::SizeBound = #heap_size;
        }

        #[doc(hidden)]
        #[allow(non_upper_case_globals)]
        impl #formula_impl_generics #ident #formula_ty_generics #formula_where_clause {
            #(
                pub const #field_order_names: usize = #fields_order;
            )*

            #field_helpers
            pub const __ALKAHEST_FIELD_COUNT: usize = #fields_count;

            #[allow(unused)]
            fn __alkahest_construct( #(#field_names: #field_tys),* ) -> Self {
                #ident { #(#field_names,)* }
            }
        }

        impl #formula_impl_generics ::alkahest::Formula<#version> for #ident #formula_ty_generics #formula_where_clause {
            type StackSize<const __SIZE_BYTES: usize> = #stack_size_ident #size_ty_generics;
            type HeapSize<const __SIZE_BYTES: usize> = #heap_size_ident #size_ty_generics;
            const INHABITED: bool = #inhabited;
            const VERSIONS: &'static [fn() -> &'static ::alkahest::Schema] = &[::alkahest::schema::<Self, #version>, #(#linked_versions),*];
            const SCHEMA: ::alkahest::Schema = ::alkahest::Schema::new::<Self, #version>(::alkahest::SchemaKind::Record(#schema_fields));
        }
    });
}

pub enum VarianFormula<T, S> {
    Unit,
    Tuple(T),
    Record(S),
}

pub fn derive_enum<T, S>(
    ident: syn::Ident,
    generics: syn::Generics,
    variants: impl Iterator<Item = (Ident, VarianFormula<T, S>)>,
    version: u32,
    linked_versions: &[TokenStream],
    field_versions: &[Vec<u32>],
    generate_type_definition: bool,
    tokens: &mut TokenStream,
) where
    T: Iterator<Item = syn::Type>,
    S: Iterator<Item = (Ident, syn::Type)>,
{
    let stack_size_ident = quote::format_ident!("__Alkahest_{}StackSize", ident);
    let heap_size_ident = quote::format_ident!("__Alkahest_{}HeapSize", ident);

    let formula_generics = make_formula_generics(&generics, generate_type_definition);
    let size_generics = make_size_generics(&formula_generics);

    let (formula_impl_generics, formula_ty_generics, formula_where_clause) =
        formula_generics.split_for_impl();

    let (size_impl_generics, size_ty_generics, size_where_clause) = size_generics.split_for_impl();

    let size_fields = SizeFields(&generics);

    // Extract information about each variant
    let variants = variants.enumerate().map(|(variant_idx, (name, variant))| {
        let field_versions = field_versions
            .get(variant_idx)
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        let variant_name = name;

        // Extract information about each field of the variant
        let (fields, stack_size, heap_size, inhabited, fields_order) = match variant {
            VarianFormula::Unit => (
                syn::Fields::Unit,
                quote::quote! { ::alkahest::SizeBound::Exact(0) },
                quote::quote! { ::alkahest::SizeBound::Exact(0) },
                VariantInhabited::True,
                None,
            ),
            VarianFormula::Tuple(tuple) => {
                // Extract information about each field
                let fields = tuple.enumerate().map(|(idx, element)| {
                    let child = field_versions.get(idx).copied().unwrap_or(0);
                    let stack_size = quote::quote! {
                        ::alkahest::stack_size_versioned::<#element, __SIZE_BYTES, #child>()
                    };

                    let heap_size = quote::quote! {
                        ::alkahest::heap_size_versioned::<#element, __SIZE_BYTES, #child>()
                    };

                    let inhabited = quote::quote! {
                        ::alkahest::inhabited_versioned::<#element, #child>()
                    };

                    let field = syn::Field {
                        attrs: Vec::new(),
                        vis: syn::Visibility::Inherited,
                        modifiers: syn::FieldModifiers::default(),
                        ident: None,
                        colon_token: None,
                        ty: element,
                        default: None,
                    };

                    (field, stack_size, heap_size, inhabited)
                });

                let (fields, field_stack_sizes, field_heap_sizes, field_inhabiteds): (
                    Punctuated<_, syn::Token![,]>,
                    Vec<_>,
                    Vec<_>,
                    Vec<_>,
                ) = fields.collect();

                let mut field_stack_sizes = field_stack_sizes.iter();
                let mut field_heap_sizes = field_heap_sizes.iter();
                let mut field_inhabiteds = field_inhabiteds.iter();

                // Generate stack size expression.
                let stack_size = match field_stack_sizes.next() {
                    None => quote::quote! {::alkahest::SizeBound::Exact(0)},
                    Some(first) => {
                        quote::quote! { #first #( .add(#field_stack_sizes) )* }
                    }
                };

                // Generate heap size expression.
                let heap_size = match field_heap_sizes.next() {
                    None => quote::quote! {::alkahest::SizeBound::Exact(0)},
                    Some(first) => {
                        quote::quote! { #first #( .add(#field_heap_sizes) )* }
                    }
                };

                // Generate inhabited expression.
                let inhabited = match field_inhabiteds.next() {
                    None => VariantInhabited::True,
                    Some(first) => {
                        VariantInhabited::Expr(quote::quote! { #first #( && #field_inhabiteds )* })
                    }
                };

                let fields = syn::Fields::Unnamed(syn::FieldsUnnamed {
                    paren_token: syn::token::Paren(Span::call_site()),
                    unnamed: fields,
                });

                (fields, stack_size, heap_size, inhabited, None)
            }
            VarianFormula::Record(record) => {
                // Extract information about each field
                let fields = record.enumerate().map(|(idx, (name, element))| {
                    let child = field_versions.get(idx).copied().unwrap_or(0);
                    let stack_size = quote::quote! {
                        ::alkahest::stack_size_versioned::<#element, __SIZE_BYTES, #child>()
                    };

                    let heap_size = quote::quote! {
                        ::alkahest::heap_size_versioned::<#element, __SIZE_BYTES, #child>()
                    };

                    let inhabited = quote::quote! {
                        ::alkahest::inhabited_versioned::<#element, #child>()
                    };

                    let field = syn::Field {
                        attrs: Vec::new(),
                        vis: syn::Visibility::Inherited,
                        modifiers: syn::FieldModifiers::default(),
                        ident: Some(name),
                        colon_token: Some(syn::Token![:](Span::call_site())),
                        ty: element,
                        default: None,
                    };

                    (field, stack_size, heap_size, inhabited)
                });

                let (fields, field_stack_sizes, field_heap_sizes, field_inhabiteds): (
                    Punctuated<_, syn::Token![,]>,
                    Vec<_>,
                    Vec<_>,
                    Vec<_>,
                ) = fields.collect();

                let mut field_stack_sizes = field_stack_sizes.iter();
                let mut field_heap_sizes = field_heap_sizes.iter();
                let mut field_inhabiteds = field_inhabiteds.iter();

                // Generate stack size expression.
                let stack_size = match field_stack_sizes.next() {
                    None => quote::quote! {::alkahest::SizeBound::Exact(0)},
                    Some(first) => {
                        quote::quote! { #first #( .add(#field_stack_sizes) )* }
                    }
                };

                // Generate heap size expression.
                let heap_size = match field_heap_sizes.next() {
                    None => quote::quote! {::alkahest::SizeBound::Exact(0)},
                    Some(first) => {
                        quote::quote! { #first #( .add(#field_heap_sizes) )* }
                    }
                };

                // Generate inhabited expression.
                let inhabited = match field_inhabiteds.next() {
                    None => VariantInhabited::True,
                    Some(first) => {
                        VariantInhabited::Expr(quote::quote! { #first #( && #field_inhabiteds )* })
                    }
                };

                let fields = syn::Fields::Named(syn::FieldsNamed {
                    brace_token: syn::token::Brace(Span::call_site()),
                    named: fields,
                });

                let field_order_names = fields.iter().map(|field| {
                    quote::format_ident!(
                        "__ALKAHEST_ORDER_OF_{}_{}",
                        variant_name,
                        field.ident.as_ref().unwrap()
                    )
                });

                let fields_order =
                    (0..fields.len()).map(|order| proc_macro2::Literal::usize_unsuffixed(order));

                let fields_order = quote::quote! {#(
                    pub const #field_order_names: usize = #fields_order;
                )*};

                (fields, stack_size, heap_size, inhabited, Some(fields_order))
            }
        };

        (
            variant_name,
            fields,
            stack_size,
            heap_size,
            inhabited,
            fields_order,
        )
    });

    let (
        variant_names,
        variant_fields,
        variant_stack_sizes,
        variant_heap_sizes,
        variant_inhabiteds,
        variant_fields_orders,
    ): (Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>) = variants.collect();

    let (variant_helpers, schema_variants): (Vec<_>, Vec<_>) = variant_names.iter().zip(&variant_fields).enumerate().map(|(variant_idx, (name, fields))| {
        let versions = field_versions.get(variant_idx).map(Vec::as_slice).unwrap_or(&[]);
        let (helpers, fields) = field_metadata(version, Some(name), fields.iter().enumerate().map(|(idx, field)| (field.ident.clone(), field.ty.clone(), versions.get(idx).copied().unwrap_or(0))));
        let tag = quote::format_ident!("__ALKAHEST_DISCRIMINANT_OF_{}", name);
        let label = name.to_string();
        (helpers, quote::quote!(::alkahest::Variant { name: #label, tag: Self::#tag, fields: #fields }))
    }).unzip();

    let discriminant_count = match variant_stack_sizes.len() {
        0 => {
            quote::quote! {0usize}
        }
        _ => {
            let base = variant_inhabiteds
                .iter()
                .filter(|inhabited| match inhabited {
                    VariantInhabited::True => true,
                    VariantInhabited::Expr(_) => false,
                })
                .count();

            let variant_inhabiteds =
                variant_inhabiteds
                    .iter()
                    .filter_map(|inhabited| match inhabited {
                        VariantInhabited::True => None,
                        VariantInhabited::Expr(expr) => {
                            Some(quote::quote! { if #expr { 1 } else { 0 } })
                        }
                    });

            if base == 0 {
                quote::quote! {{
                    // Count number of inhabitated variants
                    let __inhabited_count: usize = #(#variant_inhabiteds)+*;

                    /// Calculates the number of bytes required to store the discriminant
                    __inhabited_count
                }}
            } else {
                quote::quote! {{
                    // Count number of inhabitated variants
                    let __inhabited_count: usize = #base #(+ #variant_inhabiteds)*;

                    /// Calculates the number of bytes required to store the discriminant
                    __inhabited_count
                }}
            }
        }
    };

    let stack_size = match variant_stack_sizes.len() {
        0 => {
            quote::quote! {::alkahest::SizeBound::Exact(0)}
        }
        _ => {
            // Take max size among inhabitated variants
            // Defaulting to Exact(0) if none are inhabitated

            let sizes = variant_inhabiteds.iter().zip(&variant_stack_sizes).map(
                |(inhabited, stack_size)| match inhabited {
                    VariantInhabited::True => quote::quote! {
                        let __size = #stack_size;
                        __max_size = match __max_size {
                            ::alkahest::private::None => ::alkahest::private::Some(__size),
                            ::alkahest::private::Some(current_max) => ::alkahest::private::Some(current_max.max(__size)),
                        };
                    },
                    VariantInhabited::Expr(expr) => quote::quote! {
                        if #expr {
                            let __size = #stack_size;
                            __max_size = match __max_size {
                                ::alkahest::private::None => ::alkahest::private::Some(__size),
                                ::alkahest::private::Some(current_max) => ::alkahest::private::Some(current_max.max(__size)),
                            };
                        }
                    },
                },
            );

            quote::quote! {{
                let mut __max_size = ::alkahest::private::None;

                #(#sizes)*

                match __max_size {
                    ::alkahest::private::None => ::alkahest::SizeBound::Exact(::alkahest::private::discriminant_size(<#ident #formula_ty_generics>::__ALKAHEST_DISCRIMINANT_COUNT)),
                    ::alkahest::private::Some(size) => size.add(::alkahest::SizeBound::Exact(::alkahest::private::discriminant_size(<#ident #formula_ty_generics>::__ALKAHEST_DISCRIMINANT_COUNT))),
                }
            }}
        }
    };

    let heap_size = match variant_heap_sizes.len() {
        0 => {
            quote::quote! {::alkahest::SizeBound::Exact(0)}
        }
        _ => {
            // Take max size among inhabitated variants

            let sizes = variant_inhabiteds.iter().zip(&variant_heap_sizes).map(
                |(inhabited, heap_size)| match inhabited {
                    VariantInhabited::True => quote::quote! {
                        let __size = #heap_size;
                        __max_size = match __max_size {
                            ::alkahest::private::None => ::alkahest::private::Some(__size),
                            ::alkahest::private::Some(current_max) => ::alkahest::private::Some(current_max.max(__size)),
                        };
                    },
                    VariantInhabited::Expr(expr) => quote::quote! {
                        if #expr {
                            let __size = #heap_size;
                            __max_size = match __max_size {
                                ::alkahest::private::None => ::alkahest::private::Some(__size),
                                ::alkahest::private::Some(current_max) => ::alkahest::private::Some(current_max.max(__size)),
                            };
                        }
                    },
                },
            );

            quote::quote! {{
                let mut __max_size = ::alkahest::private::None;

                #(#sizes)*

                match __max_size {
                    ::alkahest::private::None => ::alkahest::SizeBound::Exact(0),
                    ::alkahest::private::Some(size) => size,
                }
            }}
        }
    };

    let mut variant_inhabited_iter = variant_inhabiteds.iter().peekable();

    let inhabited = match variant_inhabited_iter.peek() {
        None => quote::quote! { false },
        Some(_) => {
            if variant_inhabited_iter
                .clone()
                .any(|inhabited| matches!(inhabited, VariantInhabited::True))
            {
                // If any variant is unconditionally inhabited, the whole enum is inhabited
                quote::quote! { true }
            } else {
                let variant_inhabited_iter =
                    variant_inhabited_iter.map(|inhabited| match inhabited {
                        VariantInhabited::True => {
                            unreachable!("This case is handled by the outer condition")
                        }
                        VariantInhabited::Expr(expr) => expr,
                    });

                quote::quote! { #((#variant_inhabited_iter))&&* }
            }
        }
    };

    let variant_discriminant_names = variant_names
        .iter()
        .map(|name| quote::format_ident!("__ALKAHEST_DISCRIMINANT_OF_{}", name));

    let variant_discriminants = (0..variant_names.len()).map(|idx| {
        let inhabited = &variant_inhabiteds[idx];

        match (inhabited, idx) {
            (VariantInhabited::True, 0) => {
                quote::quote! {0usize}
            }
            (VariantInhabited::Expr(expr), 0) => {
                quote::quote! {if #expr { 0 } else { usize::MAX }}
            }
            _ => {
                let base = variant_inhabiteds[..idx]
                    .iter()
                    .filter(|inhabited| match inhabited {
                        VariantInhabited::True => true,
                        VariantInhabited::Expr(_) => false,
                    })
                    .count();

                let variant_inhabiteds =
                    variant_inhabiteds[..idx]
                        .iter()
                        .filter_map(|inhabited| match inhabited {
                            VariantInhabited::True => None,
                            VariantInhabited::Expr(expr) => {
                                Some(quote::quote! { if #expr { 1 } else { 0 } })
                            }
                        });

                match (inhabited, base) {
                    (VariantInhabited::True, 0) => {
                        quote::quote! {{
                            let __inhabited_predecessors: usize = #(#variant_inhabiteds)+*;
                            __inhabited_predecessors
                        }}
                    }
                    (VariantInhabited::True, _) => {
                        quote::quote! {{
                            let __inhabited_predecessors: usize = #base #(+ #variant_inhabiteds)*;
                            __inhabited_predecessors
                        }}
                    }
                    (VariantInhabited::Expr(expr), 0) => {
                        quote::quote! {{
                            if #expr {
                                let __inhabited_predecessors: usize = #(#variant_inhabiteds)+*;
                                __inhabited_predecessors
                            } else {
                                usize::MAX
                            }
                        }}
                    }
                    (VariantInhabited::Expr(expr), _) => {
                        quote::quote! {{
                            if #expr {
                                let __inhabited_predecessors: usize = #base #(+ #variant_inhabiteds)*;
                                __inhabited_predecessors
                            } else {
                                usize::MAX
                            }
                        }}
                    }
                }
            }
        }
    });

    if generate_type_definition {
        tokens.extend(quote::quote! {
            pub enum #ident #generics {#(
                #variant_names #variant_fields,
            )*}
        });
    }

    tokens.extend(quote::quote! {
        #[doc(hidden)]
        #[allow(non_upper_case_globals)]
        impl #formula_impl_generics #ident #formula_ty_generics #formula_where_clause {
            #(
                pub const #variant_discriminant_names: usize = #variant_discriminants;
            )*

            #(#variant_fields_orders)*
            #(#variant_helpers)*

            pub const __ALKAHEST_DISCRIMINANT_COUNT: usize = #discriminant_count;
        }

        #[allow(non_camel_case_types)]
        pub struct #stack_size_ident #size_generics #size_fields;

        impl #size_impl_generics ::alkahest::SizeType for #stack_size_ident #size_ty_generics #size_where_clause {
            const VALUE: ::alkahest::SizeBound = #stack_size;
        }

        #[allow(non_camel_case_types)]
        pub struct #heap_size_ident #size_generics #size_fields;

        impl #size_impl_generics ::alkahest::SizeType for #heap_size_ident #size_ty_generics #size_where_clause {
            const VALUE: ::alkahest::SizeBound = #heap_size;
        }

        impl #formula_impl_generics ::alkahest::Formula<#version> for #ident #formula_ty_generics #formula_where_clause {
            type StackSize<const __SIZE_BYTES: usize> = #stack_size_ident #size_ty_generics;
            type HeapSize<const __SIZE_BYTES: usize> = #heap_size_ident #size_ty_generics;
            const INHABITED: bool = #inhabited;
            const VERSIONS: &'static [fn() -> &'static ::alkahest::Schema] = &[::alkahest::schema::<Self, #version>, #(#linked_versions),*];
            const SCHEMA: ::alkahest::Schema = ::alkahest::Schema::new::<Self, #version>(::alkahest::SchemaKind::Enum { variants: &[#(#schema_variants),*], discriminant_count: Self::__ALKAHEST_DISCRIMINANT_COUNT });
        }
    });
}

pub(crate) fn derive(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(item as syn::DeriveInput);

    let args = match get_args(&input.attrs) {
        Ok(args) => FormulaArgs::from_derive(args),
        Err(err) => {
            return proc_macro::TokenStream::from(err.to_compile_error());
        }
    };

    match derive_impl(input, args) {
        Ok(tokens) => tokens.into(),
        Err(err) => proc_macro::TokenStream::from(err.to_compile_error()),
    }
}

fn get_args(attrs: &[syn::Attribute]) -> syn::Result<FormulaDeriveArgs> {
    crate::args::derive_args(attrs)
}

pub fn derive_impl(
    input: syn::DeriveInput,
    args: FormulaArgs,
) -> syn::Result<proc_macro2::TokenStream> {
    let version = crate::args::version_value(&args.version)?;
    let linked_versions = crate::args::revision_links(&args.versions, version)?;
    validate_annotations(&input.data)?;
    let field_groups: Vec<_> = match &input.data {
        syn::Data::Struct(data) => vec![&data.fields],
        syn::Data::Enum(data) => data.variants.iter().map(|v| &v.fields).collect(),
        syn::Data::Union(_) => Vec::new(),
    };
    let field_versions = field_groups
        .iter()
        .map(|fields| {
            fields
                .iter()
                .map(|field| crate::args::field_version(&field.attrs))
                .collect::<syn::Result<Vec<_>>>()
        })
        .collect::<syn::Result<Vec<_>>>()?;
    let field_bounds: Vec<syn::WherePredicate> = field_groups
        .iter()
        .zip(&field_versions)
        .flat_map(|(fields, versions)| {
            fields
                .iter()
                .zip(versions)
                .filter(|(field, _)| needs_field_bound(&field.ty, &input.generics, &input.ident))
                .map(|(field, child)| {
                    let ty = &field.ty;
                    syn::parse_quote!(#ty: ::alkahest::Element<#child>)
                })
        })
        .collect();

    let mut tokens = TokenStream::new();

    let mut generics = input.generics;
    generics.make_where_clause().predicates.extend(field_bounds);

    if args.where_clause.is_some() {
        generics
            .make_where_clause()
            .predicates
            .extend(args.where_clause.unwrap().predicates.into_iter());
    }

    match input.data {
        syn::Data::Union(union) => Err(syn::Error::new_spanned(
            union.union_token,
            "Alkahest does not support unions",
        )),
        syn::Data::Struct(data) => match data.fields {
            syn::Fields::Unit => {
                derive_unit(
                    input.ident,
                    generics,
                    version,
                    &linked_versions,
                    false,
                    &mut tokens,
                );
                Ok(tokens)
            }
            syn::Fields::Unnamed(fields) => {
                derive_tuple(
                    input.ident,
                    generics,
                    fields.unnamed.into_iter().map(|f| f.ty),
                    version,
                    &linked_versions,
                    &field_versions[0],
                    false,
                    &mut tokens,
                );
                Ok(tokens)
            }
            syn::Fields::Named(fields) => {
                derive_record(
                    input.ident,
                    generics,
                    fields.named.into_iter().map(|f| (f.ident.unwrap(), f.ty)),
                    version,
                    &linked_versions,
                    &field_versions[0],
                    false,
                    &mut tokens,
                );
                Ok(tokens)
            }
        },
        syn::Data::Enum(data) => {
            derive_enum(
                input.ident,
                generics,
                data.variants.into_iter().map(|variant| {
                    let name = variant.ident;
                    match variant.fields {
                        syn::Fields::Unit => (name, VarianFormula::Unit),
                        syn::Fields::Unnamed(fields) => (
                            name,
                            VarianFormula::Tuple(fields.unnamed.into_iter().map(|f| f.ty)),
                        ),
                        syn::Fields::Named(fields) => (
                            name,
                            VarianFormula::Record(
                                fields.named.into_iter().map(|f| (f.ident.unwrap(), f.ty)),
                            ),
                        ),
                    }
                }),
                version,
                &linked_versions,
                &field_versions,
                false,
                &mut tokens,
            );
            Ok(tokens)
        }
    }
}
