//! Generates Rust module from Alkahest module.

use alkahest_parse::{
    Element, ElementKind, Formula, List, Module, NamedElement, NamedVariant, Record, Symbol, Tuple,
    Variant, Variants,
};
use proc_macro2::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream};
use quote::TokenStreamExt;

pub fn into_ident(name: &str) -> proc_macro2::Ident {
    proc_macro2::Ident::new(name, Span::call_site())
}

pub fn symbol_to_tokens(symbol: &Symbol, tokens: &mut TokenStream) {
    let name = symbol.name();
    let path = symbol.path();

    for seg in path {
        tokens.append(into_ident(seg));
        tokens.append(Punct::new(':', Spacing::Joint));
        tokens.append(Punct::new(':', Spacing::Alone));
    }

    tokens.append(into_ident(name));
}

pub fn element_to_tokens(element: &Element, tokens: &mut TokenStream) {
    if element.indirect {
        tokens.append(Ident::new("__Alkahest_Element_Indirect", Span::call_site()));
    } else {
        tokens.append(Ident::new("__Alkahest_Element", Span::call_site()));
    }

    tokens.append(Punct::new('<', Spacing::Alone));
    match &element.kind {
        ElementKind::Symbol(symbol) => {
            symbol_to_tokens(symbol, tokens);
        }
        ElementKind::List(list) => {
            list_to_tokens(list, tokens);
        }
        ElementKind::Tuple(tup) => {
            tuple_to_tokens(tup, tokens);
        }
    }

    tokens.append(Punct::new('>', Spacing::Alone));
}

pub fn list_to_tokens(list: &List, tokens: &mut TokenStream) {
    tokens.append(Ident::new("__Alkahest__List", Span::call_site()));
    tokens.append(Punct::new('<', Spacing::Alone));
    element_to_tokens(&list.element, tokens);
    tokens.append(Punct::new(',', Spacing::Alone));
    tokens.append(Literal::u32_unsuffixed(list.min_len));
    tokens.append(Punct::new(',', Spacing::Alone));
    tokens.append(Literal::u32_unsuffixed(list.max_len));
    tokens.append(Punct::new('>', Spacing::Alone));
}

pub fn tuple_to_tokens(tuple: &Tuple, tokens: &mut TokenStream) {
    let mut inner = TokenStream::new();

    for element in tuple.elements.iter() {
        element_to_tokens(element, &mut inner);
        inner.append(Punct::new(',', Spacing::Alone));
    }

    tokens.append(Group::new(Delimiter::Parenthesis, inner));
}

pub fn record_to_tokens(record: &Record, tokens: &mut TokenStream) {
    let mut inner = TokenStream::new();

    for named_element in record.fields.iter() {
        let NamedElement { name, element } = named_element;

        inner.append(into_ident(name.as_str()));
        inner.append(Punct::new(':', Spacing::Alone));
        element_to_tokens(element, &mut inner);
        inner.append(Punct::new(',', Spacing::Alone));
    }

    tokens.append(Group::new(Delimiter::Brace, inner));
}

pub fn variant_to_tokens(variant: &Variant, tokens: &mut TokenStream) {
    match variant {
        Variant::Unit => {
            // Nothing to do
        }
        Variant::Tuple(tup) => {
            let mut inner = TokenStream::new();

            for element in tup.elements.iter() {
                element_to_tokens(element, &mut inner);
                inner.append(Punct::new(',', Spacing::Alone));
            }

            tokens.append(Group::new(Delimiter::Parenthesis, inner));
        }
        Variant::Record(record) => {
            let mut inner = TokenStream::new();

            for named_element in record.fields.iter() {
                let NamedElement { name, element } = named_element;

                inner.append(into_ident(name.as_str()));
                inner.append(Punct::new(':', Spacing::Alone));
                element_to_tokens(element, &mut inner);
                inner.append(Punct::new(',', Spacing::Alone));
            }

            tokens.append(Group::new(Delimiter::Brace, inner));
        }
    }
}

pub fn variants_to_tokens(variants: &Variants, tokens: &mut TokenStream) {
    let mut inner = TokenStream::new();

    for named_variant in variants.variants.iter() {
        let NamedVariant { name, variant } = named_variant;

        inner.append(into_ident(name.as_str()));
        variant_to_tokens(variant, &mut inner);
        inner.append(Punct::new(',', Spacing::Alone));
    }

    tokens.append(Group::new(Delimiter::Brace, inner));
}

pub fn formula_to_tokens(name: &str, formula: &Formula, tokens: &mut TokenStream) {
    match formula {
        Formula::Tuple(tuple) => {
            tokens.append(Ident::new("pub", Span::call_site()));
            tokens.append(Ident::new("struct", Span::call_site()));
            tokens.append(into_ident(name));
            tuple_to_tokens(tuple, tokens);
            tokens.append(Punct::new(';', Spacing::Alone));
        }
        Formula::Record(record) => {
            tokens.append(Ident::new("pub", Span::call_site()));
            tokens.append(Ident::new("struct", Span::call_site()));
            tokens.append(into_ident(name));
            record_to_tokens(record, tokens);
        }
        Formula::Variants(variants) => {
            tokens.append(Ident::new("pub", Span::call_site()));
            tokens.append(Ident::new("enum", Span::call_site()));
            tokens.append(into_ident(name));
            variants_to_tokens(variants, tokens);
        }
    }
}

pub fn module_to_tokens(module: &Module, tokens: &mut TokenStream) {
    for named_formula in module.formulas.iter() {
        formula_to_tokens(named_formula.name.as_str(), &named_formula.formula, tokens);
    }
}
