use proc_easy::{EasyArgument, EasyAttributes, EasyPeek, EasyToken};
use proc_macro2::Span;
use syn::{
    parse::{Lookahead1, Parse, ParseStream},
    spanned::Spanned,
};

proc_easy::easy_token!(noref);
proc_easy::easy_token!(serialize);
proc_easy::easy_token!(deserialize);
proc_easy::easy_token!(non_exhaustive);

proc_easy::easy_parse! {
    struct SchemaParams {
        token: syn::Token![for],
        generics: syn::Generics,
    }
}

struct SchemaRef {
    params: Option<SchemaParams>,
    ty: syn::Type,
    where_clause: Option<syn::WhereClause>,
}

impl From<SchemaRef> for Schema {
    fn from(schema: SchemaRef) -> Self {
        let mut generics = schema
            .params
            .map(|params| params.generics)
            .unwrap_or_default();

        if let Some(where_clause) = schema.where_clause {
            generics.make_where_clause().predicates = where_clause.predicates;
        }

        Self {
            ty: schema.ty,
            generics,
        }
    }
}

impl EasyToken for SchemaRef {
    fn display() -> &'static str {
        "Schema type"
    }
}

impl EasyPeek for SchemaRef {
    fn peek_stream(stream: ParseStream) -> bool {
        stream.peek(syn::Token![for]) || stream.peek(syn::Token![<]) || stream.peek(syn::Ident)
    }

    fn peek(lookahead1: &Lookahead1) -> bool {
        lookahead1.peek(syn::Token![for])
            || lookahead1.peek(syn::Token![<])
            || lookahead1.peek(syn::Ident)
    }
}

impl Parse for SchemaRef {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let params = if input.peek(syn::Token![for]) {
            Some(input.parse()?)
        } else {
            None
        };

        let ty = input.parse()?;

        let where_clause = if input.peek(syn::Token![where]) {
            Some(input.parse()?)
        } else {
            None
        };

        Ok(Self {
            params: params.into(),
            ty,
            where_clause,
        })
    }
}

impl Spanned for SchemaRef {
    fn span(&self) -> Span {
        let mut span = self.ty.span();

        if let Some(params) = &self.params {
            span = params.token.span().join(span).unwrap_or(span);
        };

        if let Some(where_clause) = &self.where_clause {
            span = span.join(where_clause.span()).unwrap_or(span);
        }

        span
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
        noref_token: noref,
        schema: Option<SchemaRef>,
    }
}

proc_easy::easy_argument_tuple! {
    struct SerializeArg {
        token: serialize,
        no_reference: Option<NoReferenceRef>,
        schema: Option<SchemaRef>,
    }
}

proc_easy::easy_argument_tuple! {
    struct DeserializeArg {
        token: deserialize,
        schema: Option<SchemaRef>,
        non_exhaustive: Option<non_exhaustive>,
    }
}

proc_easy::easy_attributes! {
    @(alkahest)
    struct Attrs {
        non_exhaustive: Option<non_exhaustive>,
        no_reference: Option<NoReferenceRef>,
        serialize: Vec<SerializeArg>,
        deserialize: Vec<DeserializeArg>,
        variant: Option<Variant>,
        schema: Option<SchemaRef>,
    }
}

#[derive(Clone)]
pub struct Schema {
    pub ty: syn::Type,
    pub generics: syn::Generics,
}

pub struct Args {
    pub non_exhaustive: Option<non_exhaustive>,
    pub no_reference: Option<Option<Schema>>,
    pub common: Option<Schema>,
    pub serialize: Option<Schema>,
    pub deserialize: Option<Schema>,
    pub variant: Option<syn::Ident>,
}

pub fn parse_attributes(attrs: &[syn::Attribute]) -> syn::Result<Args> {
    let attrs = Attrs::parse(attrs, Span::call_site())?;

    let mut serialize_opt = None;
    let mut deserialize_opt = None;
    let common_opt = attrs.schema.map(Schema::from);
    let mut non_exhaustive_opt = attrs.non_exhaustive;
    let mut no_reference_opt = attrs.no_reference;

    for serialize in attrs.serialize {
        if let Some(schema) = serialize.schema {
            if common_opt.is_some() {
                return Err(syn::Error::new(
                    schema.span(),
                    "Common schema reference already specified",
                ));
            }
            serialize_opt = Some(Schema::from(schema));
        }

        if let Some(no_reference) = serialize.no_reference {
            if no_reference_opt.is_some() {
                return Err(syn::Error::new(
                    no_reference.name_span(),
                    "Reference already specified",
                ));
            }

            no_reference_opt = Some(no_reference);
        }
    }

    for deserialize in attrs.deserialize {
        if let Some(schema) = deserialize.schema {
            if common_opt.is_some() {
                return Err(syn::Error::new(
                    schema.span(),
                    "Common schema reference already specified",
                ));
            }
            deserialize_opt = Some(Schema::from(schema));
        }

        if let Some(non_exhaustive) = deserialize.non_exhaustive {
            if non_exhaustive_opt.is_some() {
                return Err(syn::Error::new(
                    non_exhaustive.span(),
                    "Non-exhaustive already specified",
                ));
            }

            non_exhaustive_opt = Some(non_exhaustive);
        }
    }

    Ok(Args {
        common: common_opt,
        serialize: serialize_opt,
        deserialize: deserialize_opt,
        non_exhaustive: non_exhaustive_opt,
        no_reference: no_reference_opt.map(|noref| noref.schema.map(Schema::from)),
        variant: attrs.variant.map(|v| v.variant),
    })
}
