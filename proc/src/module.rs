use alkahest_parse::{
    Definition, Element, ElementKind, Formula, ImportTree, List, Module, Path, Tuple, Variant,
    Variants,
};
use proc_easy::EasyBraced;
use proc_macro2::{Delimiter, Group, Ident, Punct, Spacing, Span, TokenStream};
use quote::{ToTokens, TokenStreamExt};
use syn::punctuated::Punctuated;

use crate::attrs::ModuleArgs;

proc_easy::easy_parse! {
    struct Empty;
}

proc_easy::easy_parse! {
    struct ModuleItem {
        mod_token: syn::Token![mod],
        ident: syn::Ident,
        semi: EasyBraced<Empty>
    }
}

pub(crate) fn alkahest(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(item as ModuleItem);

    match alkahest_impl(attr, input) {
        Ok(tokens) => tokens.into(),
        Err(err) => proc_macro::TokenStream::from(err.to_compile_error()),
    }
}

fn alkahest_impl(
    attr: proc_macro::TokenStream,
    input: ModuleItem,
) -> syn::Result<proc_macro2::TokenStream> {
    let args = syn::parse::<ModuleArgs>(attr)?;

    let module_relative_path = match &args.path {
        None => input.ident.to_string() + ".alk",
        Some(path) => path.module.value(),
    };

    let source_path = proc_macro::Span::call_site().local_file().ok_or_else(|| {
        syn::Error::new(
            input.ident.span(),
            "Cannot determine the path of the source file",
        )
    })?;

    let base_path = source_path.parent().ok_or_else(|| {
        syn::Error::new(
            input.ident.span(),
            "Cannot determine the directory of the source file",
        )
    })?;

    let module_path = base_path.join(&module_relative_path);

    let module_source = std::fs::read_to_string(&*module_path).map_err(|err| {
        syn::Error::new(
            input.ident.span(),
            format!("Failed to read module file: {}", err),
        )
    })?;

    let module = alkahest_parse::parse_module(module_source).map_err(|err| {
        syn::Error::new(
            input.ident.span(),
            format!("Failed to parse module file: {}", err),
        )
    })?;

    let mut output = proc_macro2::TokenStream::new();

    input.mod_token.to_tokens(&mut output);
    input.ident.to_tokens(&mut output);

    syn::token::Brace::default().surround(&mut output, |tokens| {
        tokens.extend(quote::quote! {
            // This forces proc-macro to recompile when the module file changes
            const MODULE_SOURCE: &'static str = include_str!(#module_relative_path);

            use alkahest_core::private::*;
        });

        module_to_tokens(&module, tokens);
    });

    Ok(output)
}

fn into_ident(name: &str) -> proc_macro2::Ident {
    proc_macro2::Ident::new(name, Span::call_site())
}

fn path_to_tokens(path: &Path) -> syn::Path {
    let name = path.name();
    let mut path = path.path().peekable();

    let leading_colon = match path.peek() {
        Some(seg) if seg.is_empty() => {
            path.next();
            Some(syn::Token![::](Span::call_site()))
        }
        _ => None,
    };

    let mut segments: Punctuated<_, _> = path
        .map(|seg| syn::PathSegment {
            ident: into_ident(seg),
            arguments: syn::PathArguments::None,
        })
        .collect();

    segments.push(syn::PathSegment {
        ident: into_ident(name),
        arguments: syn::PathArguments::None,
    });

    syn::Path {
        leading_colon,
        segments,
    }
}

fn list_to_tokens(list: &List) -> syn::Path {
    let args = if list.min_len == list.max_len {
        [
            syn::GenericArgument::Type(element_to_tokens(&list.element)),
            syn::GenericArgument::Const(syn::Expr::Lit(syn::ExprLit {
                attrs: Vec::new(),
                lit: syn::Lit::Int(syn::LitInt::new(
                    &list.min_len.to_string(),
                    Span::call_site(),
                )),
            })),
        ]
        .into_iter()
        .collect()
    } else {
        [
            syn::GenericArgument::Type(element_to_tokens(&list.element)),
            syn::GenericArgument::Const(syn::Expr::Lit(syn::ExprLit {
                attrs: Vec::new(),
                lit: syn::Lit::Int(syn::LitInt::new(
                    &list.min_len.to_string(),
                    Span::call_site(),
                )),
            })),
            syn::GenericArgument::Const(syn::Expr::Lit(syn::ExprLit {
                attrs: Vec::new(),
                lit: syn::Lit::Int(syn::LitInt::new(
                    &list.max_len.to_string(),
                    Span::call_site(),
                )),
            })),
        ]
        .into_iter()
        .collect()
    };

    syn::Path {
        leading_colon: None,
        segments: std::iter::once(syn::PathSegment {
            ident: if list.min_len == list.max_len {
                into_ident("__Alkahest_Array")
            } else {
                into_ident("__Alkahest_List")
            },
            arguments: syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                colon2_token: None,
                lt_token: syn::Token![<](Span::call_site()),
                args,
                gt_token: syn::Token![>](Span::call_site()),
            }),
        })
        .collect(),
    }
}

fn tuple_to_tokens(tuple: &Tuple) -> syn::TypeTuple {
    let elements = tuple
        .elements
        .iter()
        .map(|element| element_to_tokens(element))
        .collect();

    syn::TypeTuple {
        paren_token: syn::token::Paren(Span::call_site()),
        elems: elements,
    }
}

fn element_to_tokens(element: &Element) -> syn::Type {
    let ty = match &element.kind {
        ElementKind::Never => syn::Type::Path(syn::TypePath {
            qself: None,
            path: syn::Path {
                leading_colon: None,
                segments: std::iter::once(syn::PathSegment {
                    ident: into_ident("__Alkahest_Never"),
                    arguments: syn::PathArguments::None,
                })
                .collect(),
            },
        }),
        ElementKind::Option(element) => syn::Type::Path(syn::TypePath {
            qself: None,
            path: syn::Path {
                leading_colon: None,
                segments: std::iter::once(syn::PathSegment {
                    ident: into_ident("__Alkahest_Option"),
                    arguments: syn::PathArguments::AngleBracketed(
                        syn::AngleBracketedGenericArguments {
                            colon2_token: None,
                            lt_token: syn::Token![<](Span::call_site()),
                            args: std::iter::once(syn::GenericArgument::Type(element_to_tokens(
                                element,
                            )))
                            .collect(),
                            gt_token: syn::Token![>](Span::call_site()),
                        },
                    ),
                })
                .collect(),
            },
        }),

        ElementKind::Symbol(symbol) => syn::Type::Path(syn::TypePath {
            qself: None,
            path: path_to_tokens(symbol),
        }),
        ElementKind::List(list) => syn::Type::Path(syn::TypePath {
            qself: None,
            path: list_to_tokens(list),
        }),
        ElementKind::Tuple(tuple) => syn::Type::Tuple(tuple_to_tokens(tuple)),
    };

    if element.indirect {
        // Wrap type in __Alkahest_Indirect< ... >
        syn::Type::Path(syn::TypePath {
            qself: None,
            path: syn::Path {
                leading_colon: None,
                segments: std::iter::once(syn::PathSegment {
                    ident: into_ident("__Alkahest_Indirect"),
                    arguments: syn::PathArguments::AngleBracketed(
                        syn::AngleBracketedGenericArguments {
                            colon2_token: None,
                            lt_token: syn::Token![<](Span::call_site()),
                            args: std::iter::once(syn::GenericArgument::Type(ty.clone())).collect(),
                            gt_token: syn::Token![>](Span::call_site()),
                        },
                    ),
                })
                .collect(),
            },
        })
    } else {
        ty
    }
}

fn make_generics(definition: &Definition) -> syn::Generics {
    let mut generics = syn::Generics::default();

    generics.gt_token = Some(syn::Token![>](Span::call_site()));
    generics.lt_token = Some(syn::Token![<](Span::call_site()));

    for param in definition.generics.iter() {
        generics.params.push(
            syn::TypeParam {
                attrs: Vec::new(),
                ident: into_ident(param.as_str()),
                colon_token: None,
                bounds: syn::punctuated::Punctuated::new(),
                eq_token: None,
                default: None,
            }
            .into(),
        );
    }

    generics
}

fn definition_to_tokens(definition: &Definition, tokens: &mut TokenStream) {
    let ident = into_ident(definition.name.as_str());
    let generics = make_generics(definition);

    let stack_size_ident = quote::format_ident!("__Alkahest_{}StackSize", ident);
    let heap_size_ident = quote::format_ident!("__Alkahest_{}HeapSize", ident);

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
                            leading_colon: None,
                            segments: std::iter::once(syn::PathSegment {
                                ident: into_ident("__Alkahest_Element"),
                                arguments: syn::PathArguments::None,
                            })
                            .collect(),
                        },
                    }))
                    .collect(),
                })
            }));
    }

    let mut size_generics = formula_generics.clone();

    size_generics.lt_token.get_or_insert_default();
    size_generics.gt_token.get_or_insert_default();

    size_generics
        .params
        .push(syn::GenericParam::Const(syn::ConstParam {
            attrs: Vec::new(),
            const_token: syn::Token![const](Span::call_site()),
            ident: into_ident("__SIZE_BYTES"),
            colon_token: syn::Token![:](Span::call_site()),
            ty: syn::Type::Path(syn::TypePath {
                qself: None,
                path: syn::Path {
                    leading_colon: None,
                    segments: std::iter::once(syn::PathSegment {
                        ident: into_ident("u8"),
                        arguments: syn::PathArguments::None,
                    })
                    .collect(),
                },
            }),
            eq_token: None,
            default: None,
        }));

    struct SizeFields<'a>(&'a syn::Generics);

    impl ToTokens for SizeFields<'_> {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            if self.0.type_params().count() == 0 {
                return;
            }

            tokens.append(Group::new(Delimiter::Parenthesis, {
                let mut tokens = TokenStream::new();

                tokens.append(Ident::new("__Alkahest_PhantomData", Span::call_site()));

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

    let size_fields = SizeFields(&size_generics);

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let (size_impl_generics, size_ty_generics, size_where_clause) = size_generics.split_for_impl();

    let (formula_impl_generics, formula_ty_generics, formula_where_clause) =
        formula_generics.split_for_impl();

    match &definition.formula {
        Formula::Unit => {
            tokens.extend(quote::quote! {
                    pub struct #ident #generics #size_fields;

                    impl #impl_generics #ident #ty_generics #where_clause {
                        const __Alkahest_FIELD_COUNT: usize = 0;
                    }

                    impl #formula_impl_generics __Alkahest_Formula for #ident #formula_ty_generics #formula_where_clause {
                        type StackSize<const __SIZE_BYTES: u8> = __Alkahest_ExactSize<0>;
                        type HeapSize<const __SIZE_BYTES: u8> = __Alkahest_ExactSize<0>;

                        const INHABITED: bool = true;
                    }
                });
        }
        Formula::Tuple(tuple) => {
            // Extract information about each field
            let fields = tuple.elements.iter().map(|element| {
                let element = element_to_tokens(element);

                let stack_size = quote::quote! {
                    __Alkahest_stack_size::<#element, __SIZE_BYTES>()
                };

                let heap_size = quote::quote! {
                    __Alkahest_heap_size::<#element, __SIZE_BYTES>()
                };

                let inhabited = quote::quote! {
                    __Alkahest_inhabited::<#element>()
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
                None => quote::quote! {__Alkahest_SizeBound::Exact(0)},
                Some(first) => {
                    quote::quote! { #first #( .add(#field_stack_sizes) )* }
                }
            };

            // Generate heap size expression.
            let heap_size = match field_heap_sizes.next() {
                None => quote::quote! {__Alkahest_SizeBound::Exact(0)},
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

            tokens.extend(quote::quote! {
                pub struct #stack_size_ident #size_generics #size_fields;

                impl #size_impl_generics __Alkahest_SizeType for #stack_size_ident #size_ty_generics #size_where_clause {
                    const VALUE: __Alkahest_SizeBound = #stack_size;
                }

                pub struct #heap_size_ident #size_generics #size_fields;

                impl #size_impl_generics __Alkahest_SizeType for #heap_size_ident #size_ty_generics #size_where_clause {
                    const VALUE: __Alkahest_SizeBound = #heap_size;
                }

                pub struct #ident #generics ( #(pub #field_tys,)* );

                impl #impl_generics #ident #ty_generics #where_clause {
                    const __Alkahest_FIELD_COUNT: usize = #fields_count;
                }

                impl #formula_impl_generics __Alkahest_Formula for #ident #formula_ty_generics #formula_where_clause {
                    type StackSize<const __SIZE_BYTES: u8> = #stack_size_ident #size_ty_generics;
                    type HeapSize<const __SIZE_BYTES: u8> = #heap_size_ident #size_ty_generics;

                    const INHABITED: bool = #inhabited;
                }
            });
        }
        Formula::Record(record) => {
            // Extract information about each field
            let fields = record.fields.iter().map(|field| {
                let element = element_to_tokens(&field.element);

                let stack_size = quote::quote! {
                    __Alkahest_stack_size::<#element, __SIZE_BYTES>()
                };

                let heap_size = quote::quote! {
                    __Alkahest_heap_size::<#element, __SIZE_BYTES>()
                };

                let inhabited = quote::quote! {
                    __Alkahest_inhabited::<#element>()
                };

                (
                    into_ident(field.name.as_str()),
                    element,
                    stack_size,
                    heap_size,
                    inhabited,
                )
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
                None => quote::quote! {__Alkahest_SizeBound::Exact(0)},
                Some(first) => {
                    quote::quote! { #first #( .add(#field_stack_sizes) )* }
                }
            };

            // Generate heap size expression.
            let heap_size = match field_heap_sizes.next() {
                None => quote::quote! {__Alkahest_SizeBound::Exact(0)},
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
                .map(|name| quote::format_ident!("__Alkahest_ORDER_OF_{}", name));

            let fields_order =
                (0..field_names.len()).map(|order| proc_macro2::Literal::usize_unsuffixed(order));

            let fields_count = field_names.len();

            tokens.extend(quote::quote! {
                pub struct #stack_size_ident #size_generics #size_fields;

                impl #size_impl_generics __Alkahest_SizeType for #stack_size_ident #size_ty_generics #size_where_clause {
                    const VALUE: __Alkahest_SizeBound = #stack_size;
                }

                pub struct #heap_size_ident #size_generics #size_fields;

                impl #size_impl_generics __Alkahest_SizeType for #heap_size_ident #size_ty_generics #size_where_clause {
                    const VALUE: __Alkahest_SizeBound = #heap_size;
                }

                pub struct #ident #generics { #(pub #field_names: #field_tys,)* }

                #[doc(hidden)]
                impl #impl_generics #ident #ty_generics #where_clause {
                    #(
                        pub const #field_order_names: usize = #fields_order;
                    )*

                    pub const __Alkahest_FIELD_COUNT: usize = #fields_count;
                }

                impl #formula_impl_generics __Alkahest_Formula for #ident #formula_ty_generics #formula_where_clause {
                    type StackSize<const __SIZE_BYTES: u8> = #stack_size_ident #size_ty_generics;
                    type HeapSize<const __SIZE_BYTES: u8> = #heap_size_ident #size_ty_generics;
                    const INHABITED: bool = #inhabited;
                }
            });
        }
        Formula::Variants(Variants(variants)) => {
            // Extract information about each variant
            let variants = variants.iter().map(|named_variant| {
                let variant_name = into_ident(named_variant.name.as_str());

                // Extract information about each field of the variant
                let (fields, stack_size, heap_size, inhabited, fields_order) =
                    match &named_variant.variant {
                        Variant::Unit => (
                            syn::Fields::Unit,
                            quote::quote! { __Alkahest_SizeBound::Exact(0) },
                            quote::quote! { __Alkahest_SizeBound::Exact(0) },
                            quote::quote! { true },
                            None,
                        ),
                        Variant::Tuple(tuple) => {
                            // Extract information about each field
                            let fields = tuple.elements.iter().map(|element| {
                                let element = element_to_tokens(element);

                                let stack_size = quote::quote! {
                                    __Alkahest_stack_size::<#element, __SIZE_BYTES>()
                                };

                                let heap_size = quote::quote! {
                                    __Alkahest_heap_size::<#element, __SIZE_BYTES>()
                                };

                                let inhabited = quote::quote! {
                                    __Alkahest_inhabited::<#element>()
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
                                None => quote::quote! {__Alkahest_SizeBound::Exact(0)},
                                Some(first) => {
                                    quote::quote! { #first #( .add(#field_stack_sizes) )* }
                                }
                            };

                            // Generate heap size expression.
                            let heap_size = match field_heap_sizes.next() {
                                None => quote::quote! {__Alkahest_SizeBound::Exact(0)},
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
                        Variant::Record(record) => {
                            // Extract information about each field
                            let fields = record.fields.iter().map(|field| {
                                let element = element_to_tokens(&field.element);

                                let stack_size = quote::quote! {
                                    __Alkahest_stack_size::<#element, __SIZE_BYTES>()
                                };

                                let heap_size = quote::quote! {
                                    __Alkahest_heap_size::<#element, __SIZE_BYTES>()
                                };

                                let inhabited = quote::quote! {
                                    __Alkahest_inhabited::<#element>()
                                };

                                let field = syn::Field {
                                    attrs: Vec::new(),
                                    vis: syn::Visibility::Inherited,
                                    mutability: syn::FieldMutability::None,
                                    ident: Some(into_ident(&field.name)),
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
                                None => quote::quote! {__Alkahest_SizeBound::Exact(0)},
                                Some(first) => {
                                    quote::quote! { #first #( .add(#field_stack_sizes) )* }
                                }
                            };

                            // Generate heap size expression.
                            let heap_size = match field_heap_sizes.next() {
                                None => quote::quote! {__Alkahest_SizeBound::Exact(0)},
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
                                    "__Alkahest_ORDER_OF_{}_{}",
                                    variant_name,
                                    field.ident.as_ref().unwrap()
                                )
                            });

                            let fields_order = (0..fields.len())
                                .map(|order| proc_macro2::Literal::usize_unsuffixed(order));

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
                    quote::quote! {__Alkahest_SizeBound::Exact(0)}
                }
                _ => {
                    // Take max size among inhabitated variants
                    // Defaulting to Exact(0) if none are inhabitated

                    quote::quote! {{
                        let mut __max_size = __Alkahest_None;

                        #(
                            if #variant_inhabiteds {
                                let __size = #variant_stack_sizes;
                                __max_size = match __max_size {
                                    __Alkahest_None => __Alkahest_Some(__size),
                                    __Alkahest_Some(current_max) => __Alkahest_Some(current_max.max(__size)),
                                };
                            }
                        )*

                        match __max_size {
                            __Alkahest_None => __Alkahest_SizeBound::Exact(__Alkahest_discriminant_size(<#ident #formula_ty_generics>::__Alkahest_DISCRIMINANT_COUNT)),
                            __Alkahest_Some(size) => size.add(__Alkahest_SizeBound::Exact(__Alkahest_discriminant_size(<#ident #formula_ty_generics>::__Alkahest_DISCRIMINANT_COUNT))),
                        }
                    }}
                }
            };

            let heap_size = match variant_heap_sizes.len() {
                0 => {
                    quote::quote! {__Alkahest_SizeBound::Exact(0)}
                }
                _ => {
                    // Take max size among inhabitated variants

                    quote::quote! {{
                        let mut __max_size = __Alkahest_None;

                        #(
                            if #variant_inhabiteds {
                                let __size = #variant_heap_sizes;
                                __max_size = match __max_size {
                                    __Alkahest_None => __Alkahest_Some(__size),
                                    __Alkahest_Some(current_max) => __Alkahest_Some(current_max.max(__size)),
                                };
                            }
                        )*

                        match __max_size {
                            __Alkahest_None => __Alkahest_SizeBound::Exact(0),
                            __Alkahest_Some(size) => size,
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
                .map(|name| quote::format_ident!("__Alkahest_DISCRIMINANT_OF_{}", name));

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

            tokens.extend(quote::quote! {

                pub enum #ident #generics {
                    #(
                        #variant_names #variant_fields,
                    )*
                }

                #[doc(hidden)]
                impl #formula_impl_generics #ident #formula_ty_generics #formula_where_clause {
                    #(
                        pub const #variant_discriminant_names: usize = #variant_discriminants;
                    )*

                    #(#variant_fields_orders)*

                    pub const __Alkahest_DISCRIMINANT_COUNT: usize = #discriminant_count;
                }

                pub struct #stack_size_ident #size_generics #size_fields;

                impl #size_impl_generics __Alkahest_SizeType for #stack_size_ident #size_ty_generics #size_where_clause {
                    const VALUE: __Alkahest_SizeBound = #stack_size;
                }

                pub struct #heap_size_ident #size_generics #size_fields;

                impl #size_impl_generics __Alkahest_SizeType for #heap_size_ident #size_ty_generics #size_where_clause {
                    const VALUE: __Alkahest_SizeBound = #heap_size;
                }

                impl #formula_impl_generics __Alkahest_Formula for #ident #formula_ty_generics #formula_where_clause {
                    type StackSize<const __SIZE_BYTES: u8> = #stack_size_ident #size_ty_generics;
                    type HeapSize<const __SIZE_BYTES: u8> = #heap_size_ident #size_ty_generics;
                    const INHABITED: bool = #inhabited;
                }
            });
        }
    }
}

fn import_to_tokens(import: &ImportTree, tokens: &mut TokenStream) {
    path_to_tokens(&import.path).to_tokens(tokens);

    if let Some(branches) = &import.branches {
        tokens.append(Punct::new(':', Spacing::Joint));
        tokens.append(Punct::new(':', Spacing::Alone));

        tokens.append(Group::new(Delimiter::Brace, {
            let mut tokens = TokenStream::new();
            for sub_import in branches.iter() {
                import_to_tokens(sub_import, &mut tokens);
                tokens.append(Punct::new(',', Spacing::Alone));
            }
            tokens
        }));
    }
}

fn module_to_tokens(module: &Module, tokens: &mut TokenStream) {
    for import in module.imports.iter() {
        tokens.append(Ident::new("use", Span::call_site()));
        import_to_tokens(import, tokens);
        tokens.append(Punct::new(';', Spacing::Alone));
    }

    for definition in module.definitions.iter() {
        definition_to_tokens(definition, tokens);
    }
}
