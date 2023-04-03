use proc_easy::{EasyArgument, EasyAttributes, EasyPeek, EasyToken};
use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::{
    parse::{Lookahead1, Parse, ParseStream},
    spanned::Spanned,
};

proc_easy::easy_token!(owned);
proc_easy::easy_token!(serialize);
proc_easy::easy_token!(deserialize);
// proc_easy::easy_token!(non_exhaustive);

proc_easy::easy_parse! {
    struct FormulaParams {
        token: syn::Token![for],
        generics: syn::Generics,
    }
}

struct FormulaRef {
    params: Option<FormulaParams>,
    path: syn::Path,
    where_clause: Option<syn::WhereClause>,
}

impl From<FormulaRef> for Formula {
    fn from(formula: FormulaRef) -> Self {
        let mut generics = formula
            .params
            .map(|params| params.generics)
            .unwrap_or_default();

        if let Some(where_clause) = formula.where_clause {
            generics.make_where_clause().predicates = where_clause.predicates;
        }

        Formula {
            path: path_make_expr_style(formula.path),
            generics,
        }
    }
}

impl EasyToken for FormulaRef {
    fn display() -> &'static str {
        "Formula type"
    }
}

impl EasyPeek for FormulaRef {
    fn peek_stream(stream: ParseStream) -> bool {
        stream.peek(syn::Token![for]) || stream.peek(syn::Token![<]) || stream.peek(syn::Ident)
    }

    fn peek(lookahead1: &Lookahead1) -> bool {
        lookahead1.peek(syn::Token![for])
            || lookahead1.peek(syn::Token![<])
            || lookahead1.peek(syn::Ident)
    }
}

impl Parse for FormulaRef {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let params = if input.peek(syn::Token![for]) {
            Some(input.parse()?)
        } else {
            None
        };

        let path = input.parse()?;

        let where_clause = if input.peek(syn::Token![where]) {
            Some(input.parse()?)
        } else {
            None
        };

        Ok(FormulaRef {
            params,
            path,
            where_clause,
        })
    }
}

impl ToTokens for FormulaRef {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if let Some(params) = &self.params {
            params.token.to_tokens(tokens);
            params.generics.to_tokens(tokens);
        }
        self.path.to_tokens(tokens);
        if let Some(where_clause) = &self.where_clause {
            where_clause.to_tokens(tokens);
        }
    }
}

proc_easy::easy_argument! {
    struct Variant {
        token: syn::Token![@],
        variant: syn::Ident,
    }
}

proc_easy::easy_argument_tuple! {
    struct NoReferenceRef {
        token: owned,
        formula: Option<FormulaRef>,
    }
}

proc_easy::easy_argument_tuple! {
    struct SerializeArg {
        token: serialize,
        owned: Option<NoReferenceRef>,
        formula: Option<FormulaRef>,
    }
}

proc_easy::easy_argument_tuple! {
    struct DeserializeArg {
        token: deserialize,
        formula: Option<FormulaRef>,
        // non_exhaustive: Option<non_exhaustive>,
    }
}

proc_easy::easy_attributes! {
    @(alkahest)
    struct Attrs {
        // non_exhaustive: Option<non_exhaustive>,
        owned: Option<NoReferenceRef>,
        serialize: Vec<SerializeArg>,
        deserialize: Vec<DeserializeArg>,
        variant: Option<Variant>,
        formula: Option<FormulaRef>,
    }
}

#[derive(Clone)]
pub struct Formula {
    pub path: syn::Path,
    pub generics: syn::Generics,
}

pub struct Args {
    // pub non_exhaustive: Option<non_exhaustive>,
    #[allow(clippy::option_option)]
    pub owned: Option<Option<Formula>>,
    pub common: Option<Formula>,
    pub serialize: Option<Formula>,
    pub deserialize: Option<Formula>,
    pub variant: Option<syn::Ident>,
}

pub fn parse_attributes(attrs: &[syn::Attribute]) -> syn::Result<Args> {
    let attrs = Attrs::parse(attrs, Span::call_site())?;

    let mut serialize_opt = None;
    let mut deserialize_opt = None;
    let common_opt = attrs.formula.map(Formula::from);
    // let mut non_exhaustive_opt = attrs.non_exhaustive;
    let mut owned_opt = attrs.owned;

    for serialize in attrs.serialize {
        if let Some(formula) = serialize.formula {
            if common_opt.is_some() {
                return Err(syn::Error::new(
                    formula.span(),
                    "Common formula reference already specified",
                ));
            }
            serialize_opt = Some(Formula::from(formula));
        }

        if let Some(owned) = serialize.owned {
            if owned_opt.is_some() {
                return Err(syn::Error::new(
                    owned.name_span(),
                    "Reference already specified",
                ));
            }

            owned_opt = Some(owned);
        }
    }

    for deserialize in attrs.deserialize {
        if let Some(formula) = deserialize.formula {
            if common_opt.is_some() {
                return Err(syn::Error::new(
                    formula.span(),
                    "Common formula reference already specified",
                ));
            }
            deserialize_opt = Some(Formula::from(formula));
        }

        // if let Some(non_exhaustive) = deserialize.non_exhaustive {
        //     if non_exhaustive_opt.is_some() {
        //         return Err(syn::Error::new(
        //             non_exhaustive.span(),
        //             "Non-exhaustive already specified",
        //         ));
        //     }

        //     non_exhaustive_opt = Some(non_exhaustive);
        // }
    }

    Ok(Args {
        common: common_opt,
        serialize: serialize_opt,
        deserialize: deserialize_opt,
        // non_exhaustive: non_exhaustive_opt,
        owned: owned_opt.map(|owned| owned.formula.map(Formula::from)),
        variant: attrs.variant.map(|v| v.variant),
    })
}

pub fn path_make_expr_style(mut path: syn::Path) -> syn::Path {
    for seg in &mut path.segments {
        if let syn::PathArguments::AngleBracketed(ref mut args) = seg.arguments {
            args.colon2_token = Some(<syn::Token![::]>::default());
        }
    }
    path
}
