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

fn make_formula_generics(generics: &syn::Generics) -> syn::Generics {
    let mut formula_generics = generics.clone();
    if generics.type_params().count() > 0 {
        formula_generics
            .make_where_clause()
            .predicates
            .extend(generics.type_params().map(|param| {
                syn::WherePredicate::Type(syn::PredicateType {
                    lifetimes: None,
                    bounded_ty: syn::Type::Path(syn::TypePath {
                        qself: None,
                        path: syn::Path {
                            leading_colon: None,
                            segments: std::iter::once(syn::PathSegment {
                                ident: param.ident.clone(),
                                arguments: syn::PathArguments::None,
                            })
                            .collect(),
                        },
                    }),
                    colon_token: syn::Token![:](Span::call_site()),
                    bounds: std::iter::once(syn::TypeParamBound::Trait(syn::TraitBound {
                        paren_token: None,
                        modifier: syn::TraitBoundModifier::None,
                        lifetimes: None,
                        path: syn::Path {
                            leading_colon: Some(syn::Token![::](Span::call_site())),
                            segments: [
                                syn::PathSegment {
                                    ident: syn::Ident::new("alkahest", Span::call_site()),
                                    arguments: syn::PathArguments::None,
                                },
                                syn::PathSegment {
                                    ident: syn::Ident::new("Element", Span::call_site()),
                                    arguments: syn::PathArguments::None,
                                },
                            ]
                            .into_iter()
                            .collect(),
                        },
                    }))
                    .collect(),
                })
            }));
    }

    formula_generics
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
                qself: None,
                path: syn::Path {
                    leading_colon: None,
                    segments: std::iter::once(syn::PathSegment {
                        ident: syn::Ident::new("u8", Span::call_site()),
                        arguments: syn::PathArguments::None,
                    })
                    .collect(),
                },
            }),
            eq_token: None,
            default: None,
        }));

    size_generics
}

pub fn derive_unit(
    ident: syn::Ident,
    generics: syn::Generics,
    generate_type_definition: bool,
    tokens: &mut TokenStream,
) {
    let formula_generics = make_formula_generics(&generics);

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

        impl #formula_impl_generics ::alkahest::Formula for #ident #formula_ty_generics #formula_where_clause {
            type StackSize<const __SIZE_BYTES: u8> = ::alkahest::ExactSize<0>;
            type HeapSize<const __SIZE_BYTES: u8> = ::alkahest::ExactSize<0>;

            const INHABITED: bool = true;
        }
    });
}

pub fn derive_tuple(
    ident: syn::Ident,
    generics: syn::Generics,
    fields: impl Iterator<Item = syn::Type>,
    generate_type_definition: bool,
    tokens: &mut TokenStream,
) {
    let stack_size_ident = quote::format_ident!("__Alkahest_{}StackSize", ident);
    let heap_size_ident = quote::format_ident!("__Alkahest_{}HeapSize", ident);

    let formula_generics = make_formula_generics(&generics);
    let size_generics = make_size_generics(&formula_generics);

    let (formula_impl_generics, formula_ty_generics, formula_where_clause) =
        formula_generics.split_for_impl();

    let (size_impl_generics, size_ty_generics, size_where_clause) = size_generics.split_for_impl();

    let size_fields = SizeFields(&generics);

    // Extract information about each field
    let fields = fields.map(|element| {
        let stack_size = quote::quote! {
            ::alkahest::stack_size::<#element, __SIZE_BYTES>()
        };

        let heap_size = quote::quote! {
            ::alkahest::heap_size::<#element, __SIZE_BYTES>()
        };

        let inhabited = quote::quote! {
            ::alkahest::inhabited::<#element>()
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
            const __ALKAHEST_FIELD_COUNT: usize = #fields_count;

            #[allow(unused)]
            fn __alkahest_construct( #(#bound_fields: #field_tys),* ) -> Self {
                #ident ( #(#bound_fields,)* )
            }
        }

        impl #formula_impl_generics ::alkahest::Formula for #ident #formula_ty_generics #formula_where_clause {
            type StackSize<const __SIZE_BYTES: u8> = #stack_size_ident #size_ty_generics;
            type HeapSize<const __SIZE_BYTES: u8> = #heap_size_ident #size_ty_generics;

            const INHABITED: bool = #inhabited;
        }
    });
}

pub fn derive_record(
    ident: syn::Ident,
    generics: syn::Generics,
    fields: impl Iterator<Item = (Ident, syn::Type)>,
    generate_type_definition: bool,
    tokens: &mut TokenStream,
) {
    let stack_size_ident = quote::format_ident!("__Alkahest_{}StackSize", ident);
    let heap_size_ident = quote::format_ident!("__Alkahest_{}HeapSize", ident);

    let formula_generics = make_formula_generics(&generics);
    let size_generics = make_size_generics(&formula_generics);

    let (formula_impl_generics, formula_ty_generics, formula_where_clause) =
        formula_generics.split_for_impl();

    let (size_impl_generics, size_ty_generics, size_where_clause) = size_generics.split_for_impl();

    let size_fields = SizeFields(&generics);

    // Extract information about each field
    let fields = fields.map(|(name, element)| {
        let stack_size = quote::quote! {
            ::alkahest::stack_size::<#element, __SIZE_BYTES>()
        };

        let heap_size = quote::quote! {
            ::alkahest::heap_size::<#element, __SIZE_BYTES>()
        };

        let inhabited = quote::quote! {
            ::alkahest::inhabited::<#element>()
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

            pub const __ALKAHEST_FIELD_COUNT: usize = #fields_count;

            #[allow(unused)]
            fn __alkahest_construct( #(#field_names: #field_tys),* ) -> Self {
                #ident { #(#field_names,)* }
            }
        }

        impl #formula_impl_generics ::alkahest::Formula for #ident #formula_ty_generics #formula_where_clause {
            type StackSize<const __SIZE_BYTES: u8> = #stack_size_ident #size_ty_generics;
            type HeapSize<const __SIZE_BYTES: u8> = #heap_size_ident #size_ty_generics;
            const INHABITED: bool = #inhabited;
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
    generate_type_definition: bool,
    tokens: &mut TokenStream,
) where
    T: Iterator<Item = syn::Type>,
    S: Iterator<Item = (Ident, syn::Type)>,
{
    let stack_size_ident = quote::format_ident!("__Alkahest_{}StackSize", ident);
    let heap_size_ident = quote::format_ident!("__Alkahest_{}HeapSize", ident);

    let formula_generics = make_formula_generics(&generics);
    let size_generics = make_size_generics(&formula_generics);

    let (formula_impl_generics, formula_ty_generics, formula_where_clause) =
        formula_generics.split_for_impl();

    let (size_impl_generics, size_ty_generics, size_where_clause) = size_generics.split_for_impl();

    let size_fields = SizeFields(&generics);

    // Extract information about each variant
    let variants = variants.map(|(name, variant)| {
        let variant_name = name;

        // Extract information about each field of the variant
        let (fields, stack_size, heap_size, inhabited, fields_order) = match variant {
            VarianFormula::Unit => (
                syn::Fields::Unit,
                quote::quote! { ::alkahest::SizeBound::Exact(0) },
                quote::quote! { ::alkahest::SizeBound::Exact(0) },
                quote::quote! { true },
                None,
            ),
            VarianFormula::Tuple(tuple) => {
                // Extract information about each field
                let fields = tuple.map(|element| {
                    let stack_size = quote::quote! {
                        ::alkahest::stack_size::<#element, __SIZE_BYTES>()
                    };

                    let heap_size = quote::quote! {
                        ::alkahest::heap_size::<#element, __SIZE_BYTES>()
                    };

                    let inhabited = quote::quote! {
                        ::alkahest::inhabited::<#element>()
                    };

                    let field = syn::Field {
                        attrs: Vec::new(),
                        vis: syn::Visibility::Inherited,
                        mutability: syn::FieldMutability::None,
                        ident: None,
                        colon_token: None,
                        ty: element,
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
                    None => quote::quote! { true },
                    Some(first) => {
                        quote::quote! { #first #( && #field_inhabiteds )* }
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
                let fields = record.map(|(name, element)| {
                    let stack_size = quote::quote! {
                        ::alkahest::stack_size::<#element, __SIZE_BYTES>()
                    };

                    let heap_size = quote::quote! {
                        ::alkahest::heap_size::<#element, __SIZE_BYTES>()
                    };

                    let inhabited = quote::quote! {
                        ::alkahest::inhabited::<#element>()
                    };

                    let field = syn::Field {
                        attrs: Vec::new(),
                        vis: syn::Visibility::Inherited,
                        mutability: syn::FieldMutability::None,
                        ident: Some(name),
                        colon_token: Some(syn::Token![:](Span::call_site())),
                        ty: element,
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
                    None => quote::quote! { true },
                    Some(first) => {
                        quote::quote! { #first #( && #field_inhabiteds )* }
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

    let discriminant_count = match variant_stack_sizes.len() {
        0 => {
            quote::quote! {0usize}
        }
        _ => {
            quote::quote! {{
                // Count number of inhabitated variants
                let __inhabited_count: usize = #(if #variant_inhabiteds { 1 } else { 0 } )+*;

                /// Calculates the number of bytes required to store the discriminant
                __inhabited_count
            }}
        }
    };

    let stack_size = match variant_stack_sizes.len() {
        0 => {
            quote::quote! {::alkahest::SizeBound::Exact(0)}
        }
        _ => {
            // Take max size among inhabitated variants
            // Defaulting to Exact(0) if none are inhabitated

            quote::quote! {{
                let mut __max_size = ::alkahest::private::None;

                #(
                    if #variant_inhabiteds {
                        let __size = #variant_stack_sizes;
                        __max_size = match __max_size {
                            ::alkahest::private::None => ::alkahest::private::Some(__size),
                            ::alkahest::private::Some(current_max) => ::alkahest::private::Some(current_max.max(__size)),
                        };
                    }
                )*

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

            quote::quote! {{
                let mut __max_size = ::alkahest::private::None;

                #(
                    if #variant_inhabiteds {
                        let __size = #variant_heap_sizes;
                        __max_size = match __max_size {
                            ::alkahest::private::None => ::alkahest::private::Some(__size),
                            ::alkahest::private::Some(current_max) => ::alkahest::private::Some(current_max.max(__size)),
                        };
                    }
                )*

                match __max_size {
                    ::alkahest::private::None => ::alkahest::SizeBound::Exact(0),
                    ::alkahest::private::Some(size) => size,
                }
            }}
        }
    };

    let mut variant_inhabited_iter = variant_inhabiteds.iter();

    let inhabited = match variant_inhabited_iter.next() {
        None => quote::quote! { false },
        Some(first) => {
            quote::quote! { (#first) #( || (#variant_inhabited_iter) )* }
        }
    };

    let variant_discriminant_names = variant_names
        .iter()
        .map(|name| quote::format_ident!("__ALKAHEST_DISCRIMINANT_OF_{}", name));

    let variant_discriminants = (0..variant_names.len()).map(|idx| {
        let inhabited = &variant_inhabiteds[idx];

        match idx {
            0 => quote::quote! { if #inhabited { 0 } else { usize::MAX } },
            _ => {
                let prev = &variant_inhabiteds[..idx];
                quote::quote! {{
                    if #inhabited {
                        #(if #prev { 1 } else { 0 })+*
                    } else {
                        usize::MAX
                    }
                }}
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

        impl #formula_impl_generics ::alkahest::Formula for #ident #formula_ty_generics #formula_where_clause {
            type StackSize<const __SIZE_BYTES: u8> = #stack_size_ident #size_ty_generics;
            type HeapSize<const __SIZE_BYTES: u8> = #heap_size_ident #size_ty_generics;
            const INHABITED: bool = #inhabited;
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
    for attr in attrs {
        if attr.path().is_ident("alkahest") {
            let args = attr.parse_args::<FormulaDeriveArgs>()?;
            return Ok(args);
        }
    }

    Ok(FormulaDeriveArgs::default())
}

pub fn derive_impl(
    input: syn::DeriveInput,
    mut args: FormulaArgs,
) -> syn::Result<proc_macro2::TokenStream> {
    args.add_default_where_clause(&input.generics);

    let mut tokens = TokenStream::new();

    let mut generics = input.generics;

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
                derive_unit(input.ident, generics, false, &mut tokens);
                Ok(tokens)
            }
            syn::Fields::Unnamed(fields) => {
                derive_tuple(
                    input.ident,
                    generics,
                    fields.unnamed.into_iter().map(|f| f.ty),
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
                false,
                &mut tokens,
            );
            Ok(tokens)
        }
    }
}
