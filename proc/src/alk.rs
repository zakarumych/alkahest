use alkahest_parse::{
    Builtin, Definition, Element, ElementKind, Formula, ImportTree, List, Module, Path, Tuple,
    Variant, Variants,
};
use proc_macro2::{Delimiter, Group, Ident, Punct, Spacing, Span, TokenStream};
use quote::{ToTokens, TokenStreamExt};
use syn::punctuated::Punctuated;

use crate::formula::{VarianFormula, derive_enum, derive_record, derive_tuple, derive_unit};

fn path_to_tokens(path: &Path) -> syn::Path {
    let name = path.name();
    let mut path = path.path().peekable();

    let (leading_colon, first_segment) = match path.peek() {
        Some(seg) if seg.is_empty() => {
            path.next();
            match path.peek() {
                Some(seg) if seg.is_empty() => {
                    path.next();
                    match path.peek() {
                        Some(seg) if seg.is_empty() => {
                            path.next();

                            match path.peek() {
                                Some(seg) if seg.is_empty() => {
                                    panic!(
                                        "Invalid path: cannot have more than 3 leading empty segments"
                                    );
                                }
                                _ => {}
                            }

                            // External path with leading `::`
                            (Some(syn::Token![::](Span::call_site())), None)
                        }

                        _ => {
                            // crate root path with leading `crate`
                            (None, Some(Ident::new("crate", Span::call_site())))
                        }
                    }
                }
                _ => {
                    // Parent path with leading `super`
                    (None, Some(Ident::new("super", Span::call_site())))
                }
            }
        }
        _ => {
            // Normal path.
            (None, None)
        }
    };

    let mut segments = Punctuated::new();
    if let Some(first_segment) = first_segment {
        segments.push(syn::PathSegment {
            ident: first_segment,
            arguments: syn::PathArguments::None,
        });
    }

    segments.extend(path.map(|seg| syn::PathSegment {
        ident: Ident::new(seg, Span::call_site()),
        arguments: syn::PathArguments::None,
    }));

    segments.push(syn::PathSegment {
        ident: Ident::new(name, Span::call_site()),
        arguments: syn::PathArguments::None,
    });

    syn::Path {
        leading_colon,
        segments,
    }
}

fn list_to_tokens(list: &List) -> syn::Path {
    syn::Path {
        leading_colon: Some(syn::Token![::](Span::call_site())),
        segments: [
            syn::PathSegment {
                ident: Ident::new("alkahest", Span::call_site()),
                arguments: syn::PathArguments::None,
            },
            syn::PathSegment {
                ident: if list.min_len == list.max_len {
                    Ident::new("Array", Span::call_site())
                } else {
                    Ident::new("List", Span::call_site())
                },
                arguments: syn::PathArguments::AngleBracketed(
                    syn::AngleBracketedGenericArguments {
                        colon2_token: None,
                        lt_token: syn::Token![<](Span::call_site()),
                        args: if list.min_len == list.max_len {
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
                        },
                        gt_token: syn::Token![>](Span::call_site()),
                    },
                ),
            },
        ]
        .into_iter()
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

fn symbol_in_alkahest(symbol: &str) -> syn::Path {
    syn::Path {
        leading_colon: Some(syn::Token![::](Span::call_site())),
        segments: [
            syn::PathSegment {
                ident: Ident::new("alkahest", Span::call_site()),
                arguments: syn::PathArguments::None,
            },
            syn::PathSegment {
                ident: Ident::new(symbol, Span::call_site()),
                arguments: syn::PathArguments::None,
            },
        ]
        .into_iter()
        .collect(),
    }
}

fn symbol_in_alkahest_private(symbol: &str) -> syn::Path {
    syn::Path {
        leading_colon: Some(syn::Token![::](Span::call_site())),
        segments: [
            syn::PathSegment {
                ident: Ident::new("alkahest", Span::call_site()),
                arguments: syn::PathArguments::None,
            },
            syn::PathSegment {
                ident: Ident::new("private", Span::call_site()),
                arguments: syn::PathArguments::None,
            },
            syn::PathSegment {
                ident: Ident::new(symbol, Span::call_site()),
                arguments: syn::PathArguments::None,
            },
        ]
        .into_iter()
        .collect(),
    }
}

fn builtin_to_tokens(builtin: Builtin) -> syn::Type {
    syn::Type::Path(syn::TypePath {
        qself: None,
        path: match builtin {
            Builtin::Never => symbol_in_alkahest("Never"),
            Builtin::Bool => symbol_in_alkahest_private("bool"),
            Builtin::U8 => symbol_in_alkahest_private("u8"),
            Builtin::U16 => symbol_in_alkahest_private("u16"),
            Builtin::U32 => symbol_in_alkahest_private("u32"),
            Builtin::U64 => symbol_in_alkahest_private("u64"),
            Builtin::U128 => symbol_in_alkahest_private("u128"),
            Builtin::I8 => symbol_in_alkahest_private("i8"),
            Builtin::I16 => symbol_in_alkahest_private("i16"),
            Builtin::I32 => symbol_in_alkahest_private("i32"),
            Builtin::I64 => symbol_in_alkahest_private("i64"),
            Builtin::I128 => symbol_in_alkahest_private("i128"),
            Builtin::F32 => symbol_in_alkahest_private("f32"),
            Builtin::F64 => symbol_in_alkahest_private("f64"),
            Builtin::String => symbol_in_alkahest("String"),
        },
    })
}
fn element_to_tokens(element: &Element) -> syn::Type {
    let ty = match &element.kind {
        ElementKind::Builtin(builtin) => builtin_to_tokens(*builtin),
        ElementKind::Option(element) => syn::Type::Path(syn::TypePath {
            qself: None,
            path: syn::Path {
                leading_colon: None,
                segments: std::iter::once(syn::PathSegment {
                    ident: Ident::new("Option", Span::call_site()),
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
        // Wrap type in alakhest::Indirect< ... >
        syn::Type::Path(syn::TypePath {
            qself: None,
            path: syn::Path {
                leading_colon: Some(syn::Token![::](Span::call_site())),
                segments: [
                    syn::PathSegment {
                        ident: Ident::new("alkahest", Span::call_site()),
                        arguments: syn::PathArguments::None,
                    },
                    syn::PathSegment {
                        ident: Ident::new("Indirect", Span::call_site()),
                        arguments: syn::PathArguments::AngleBracketed(
                            syn::AngleBracketedGenericArguments {
                                colon2_token: None,
                                lt_token: syn::Token![<](Span::call_site()),
                                args: std::iter::once(syn::GenericArgument::Type(ty.clone()))
                                    .collect(),
                                gt_token: syn::Token![>](Span::call_site()),
                            },
                        ),
                    },
                ]
                .into_iter()
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
                ident: Ident::new(param, Span::call_site()),
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

pub fn definition_to_tokens(definition: &Definition, tokens: &mut TokenStream) {
    let ident = Ident::new(definition.name.as_str(), Span::call_site());
    let generics = make_generics(definition);

    match &definition.formula {
        Formula::Unit => {
            derive_unit(ident, generics, true, tokens);
        }
        Formula::Tuple(tuple) => {
            derive_tuple(
                ident,
                generics,
                tuple.elements.iter().map(|e| element_to_tokens(e)),
                true,
                tokens,
            );
        }
        Formula::Record(record) => {
            derive_record(
                ident,
                generics,
                record.fields.iter().map(|f| {
                    (
                        Ident::new(&f.name, Span::call_site()),
                        element_to_tokens(&f.element),
                    )
                }),
                true,
                tokens,
            );
        }
        Formula::Variants(Variants(variants)) => {
            derive_enum(
                ident,
                generics,
                variants.iter().map(|named_variant| {
                    let name = Ident::new(&named_variant.name, Span::call_site());
                    let kind = match &named_variant.variant {
                        Variant::Unit => VarianFormula::Unit,
                        Variant::Tuple(tuple) => VarianFormula::Tuple(
                            tuple.elements.iter().map(|e| element_to_tokens(e)),
                        ),
                        Variant::Record(record) => {
                            VarianFormula::Record(record.fields.iter().map(|f| {
                                (
                                    Ident::new(&f.name, Span::call_site()),
                                    element_to_tokens(&f.element),
                                )
                            }))
                        }
                    };
                    (name, kind)
                }),
                true,
                tokens,
            );
        }
    }
}

pub fn import_to_tokens(import: &ImportTree, tokens: &mut TokenStream) {
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

pub fn module_to_tokens(module: &Module, tokens: &mut TokenStream) {
    for import in module.imports.iter() {
        tokens.append(Ident::new("use", Span::call_site()));
        import_to_tokens(import, tokens);
        tokens.append(Punct::new(';', Spacing::Alone));
    }

    for definition in module.definitions.iter() {
        definition_to_tokens(definition, tokens);
    }
}

pub fn module_from_path(
    path: &std::path::Path,
    span: Span,
) -> Result<alkahest_parse::Module, syn::Error> {
    let source_path = proc_macro::Span::call_site()
        .local_file()
        .ok_or_else(|| syn::Error::new(span, "Cannot determine the path of the source file"))?;

    let base_path = source_path.parent().ok_or_else(|| {
        syn::Error::new(span, "Cannot determine the directory of the source file")
    })?;

    let module_path = base_path.join(path);

    let module_source = std::fs::read_to_string(&*module_path)
        .map_err(|err| syn::Error::new(span, format!("Failed to read module file: {}", err)))?;

    let module = alkahest_parse::parse_module(module_source)
        .map_err(|err| syn::Error::new(span, format!("Failed to parse module file: {}", err)))?;

    Ok(module)
}
