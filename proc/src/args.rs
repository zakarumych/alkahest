use proc_easy::{EasyMaybe, EasyPeek};
use proc_macro2::Span;
use syn::{GenericParam, Ident, WherePredicate, parse::Lookahead1, token::Comma};

proc_easy::easy_token!(path);
proc_easy::easy_token!(formula);
proc_easy::easy_token!(Formula);
proc_easy::easy_token!(Serialize);
proc_easy::easy_token!(Deserialize);
proc_easy::easy_token!(Mixture);

proc_easy::easy_argument_value! {
    pub struct ModulePath {
        path: path,
        pub module: syn::LitStr,
    }
}

proc_easy::easy_terminated! {
    @(Comma)
    pub struct ModuleArgs {
        pub path: Option<ModulePath>,
    }
}

proc_easy::easy_parse! {
    pub struct Variant {
        pub at: syn::Token![@],
        pub ident: syn::Ident,
    }
}

pub struct ForGenerics {
    pub _for: syn::Token![for],
    pub _lt: syn::Token![<],
    pub params: Vec<GenericParam>,
    pub _gt: syn::Token![>],
}

impl EasyPeek for ForGenerics {
    fn peek(lookahead1: &Lookahead1) -> bool {
        lookahead1.peek(syn::Token![for])
    }

    fn peek_stream(stream: syn::parse::ParseStream) -> bool {
        stream.peek(syn::Token![for])
    }
}

impl syn::parse::Parse for ForGenerics {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let _for = input.parse()?;
        let _lt = input.parse()?;

        let param = input.parse::<GenericParam>()?;
        let mut params = vec![param];

        while !input.peek(syn::Token![>]) {
            let _comma = input.parse::<Comma>()?;
            let param = input.parse::<GenericParam>()?;
            params.push(param);
        }

        let _gt = input.parse()?;

        Ok(ForGenerics {
            _for,
            _lt,
            params,
            _gt,
        })
    }
}

proc_easy::easy_parse! {
    #[derive(Default)]
    pub struct FormulaDeriveArgs {
        pub where_clause: Option<syn::WhereClause>,
    }
}

proc_easy::easy_parse! {
    pub struct SerializeDeriveArgs {
        pub formula: syn::Path,
        pub variant: proc_easy::EasyMaybe<Variant>,
        pub where_clause: Option<syn::WhereClause>,
    }
}

impl Default for SerializeDeriveArgs {
    fn default() -> Self {
        SerializeDeriveArgs {
            formula: syn::Path::from(syn::Ident::new("Self", Span::call_site())),
            variant: proc_easy::EasyMaybe::Nothing,
            where_clause: None,
        }
    }
}

proc_easy::easy_parse! {
    pub struct DeserializeDeriveArgs {
        pub formula: syn::Path,
        pub where_clause: Option<syn::WhereClause>,
    }
}

impl Default for DeserializeDeriveArgs {
    fn default() -> Self {
        DeserializeDeriveArgs {
            formula: syn::Path::from(syn::Ident::new("Self", Span::call_site())),
            where_clause: None,
        }
    }
}

proc_easy::easy_parse! {
    #[derive(Default)]
    pub struct MixtureDeriveArgs {
        pub where_clause: Option<syn::WhereClause>,
    }
}

proc_easy::easy_parse! {
    pub struct FormulaArgs {
        pub _formula: Formula,
        pub where_clause: Option<syn::WhereClause>,
    }
}

proc_easy::easy_parse! {
    pub struct SerializeArgs {
        pub params: EasyMaybe<ForGenerics>,
        pub _serialize: Serialize,
        pub _lt: syn::Token![<],
        pub formula: syn::Path,
        pub _gt: syn::Token![>],
        pub variant: proc_easy::EasyMaybe<Variant>,
        pub where_clause: Option<syn::WhereClause>,
    }
}

proc_easy::easy_parse! {
    pub struct DeserializeLifetime {
        pub lifetime: syn::Lifetime,
        pub _comma: syn::Token![,],
    }
}

proc_easy::easy_parse! {
    pub struct DeserializeArgs {
        pub params: EasyMaybe<ForGenerics>,
        pub _deserialize: Deserialize,
        pub _lt: syn::Token![<],
        pub lifetime: EasyMaybe<DeserializeLifetime>,
        pub formula: syn::Path,
        pub _gt: syn::Token![>],
        pub where_clause: Option<syn::WhereClause>,
    }
}

proc_easy::easy_parse! {
    pub struct MixtureArgs {
        pub _mixture: Mixture,
        pub where_clause: Option<syn::WhereClause>,
    }
}

pub enum TypeArgs {
    Formula(FormulaArgs),
    Serialize(SerializeArgs),
    Deserialize(DeserializeArgs),
    Mixture(MixtureArgs),
}

impl syn::parse::Parse for TypeArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead1 = input.lookahead1();

        if lookahead1.peek(Formula) {
            let formula = input.parse::<FormulaArgs>()?;
            return Ok(TypeArgs::Formula(formula));
        }

        if lookahead1.peek(Serialize) {
            let serialize = input.parse::<SerializeArgs>()?;
            return Ok(TypeArgs::Serialize(serialize));
        }

        if lookahead1.peek(Deserialize) {
            let deserialize = input.parse::<DeserializeArgs>()?;
            return Ok(TypeArgs::Deserialize(deserialize));
        }

        if lookahead1.peek(Mixture) {
            let mixture = input.parse::<MixtureArgs>()?;
            return Ok(TypeArgs::Mixture(mixture));
        }

        if lookahead1.peek(syn::Token![for]) {
            let params = input.parse::<EasyMaybe<ForGenerics>>()?;

            let lookahead1 = input.lookahead1();

            if lookahead1.peek(Serialize) {
                let args = input.parse::<SerializeArgs>()?;
                return Ok(TypeArgs::Serialize(SerializeArgs { params, ..args }));
            }

            if lookahead1.peek(Deserialize) {
                let args = input.parse::<DeserializeArgs>()?;
                return Ok(TypeArgs::Deserialize(DeserializeArgs { params, ..args }));
            }

            return Err(lookahead1.error());
        }

        Err(lookahead1.error())
    }
}

fn merge_generics(
    base: &syn::Generics,
    additional: &EasyMaybe<ForGenerics>,
    where_clause: &Option<syn::WhereClause>,
) -> syn::Generics {
    let mut generics = base.clone();

    if let EasyMaybe::Just(additional) = additional {
        for param in additional.params.iter() {
            generics.params.push(param.clone());
        }
    }

    if let Some(where_clause) = where_clause
        && !where_clause.predicates.is_empty()
    {
        generics
            .make_where_clause()
            .predicates
            .extend(where_clause.predicates.clone());
    }

    generics
}

// fn impl_for_each_type_argument(
//     arguments: &syn::AngleBracketedGenericArguments,
//     bound: &str,
// ) -> Option<syn::WhereClause> {
//     let type_arguments = arguments.args.iter().filter_map(|arg| match arg {
//         syn::GenericArgument::Type(arg) => Some(arg),
//         _ => None,
//     });

//     type_arguments.clone().next()?;

//     Some(syn::WhereClause {
//         where_token: syn::Token![where](Span::call_site()),
//         predicates: type_arguments
//             .map(|ty| {
//                 WherePredicate::Type(syn::PredicateType {
//                     lifetimes: None,
//                     bounded_ty: ty.clone(),
//                     colon_token: syn::Token![:](Span::call_site()),
//                     bounds: std::iter::once(syn::TypeParamBound::Trait(syn::TraitBound {
//                         paren_token: None,
//                         modifier: syn::TraitBoundModifier::None,
//                         lifetimes: None,
//                         path: syn::Path {
//                             leading_colon: Some(syn::Token![::](Span::call_site())),
//                             segments: [
//                                 syn::PathSegment {
//                                     ident: Ident::new("alkahest", Span::call_site()),
//                                     arguments: syn::PathArguments::None,
//                                 },
//                                 syn::PathSegment {
//                                     ident: Ident::new(bound, Span::call_site()),
//                                     arguments: syn::PathArguments::None,
//                                 },
//                             ]
//                             .into_iter()
//                             .collect(),
//                         },
//                     }))
//                     .collect(),
//                 })
//             })
//             .collect(),
//     })
// }

fn impl_for_each_type_param(generics: &syn::Generics, bound: &str) -> Option<syn::WhereClause> {
    generics.type_params().next()?;

    Some(syn::WhereClause {
        where_token: syn::Token![where](Span::call_site()),
        predicates: generics
            .type_params()
            .map(|param| {
                WherePredicate::Type(syn::PredicateType {
                    lifetimes: None,
                    bounded_ty: syn::TypePath {
                        qself: None,
                        path: syn::Path::from(param.ident.clone()),
                    }
                    .into(),
                    colon_token: syn::Token![:](Span::call_site()),
                    bounds: std::iter::once(syn::TypeParamBound::Trait(syn::TraitBound {
                        paren_token: None,
                        modifier: syn::TraitBoundModifier::None,
                        lifetimes: None,
                        path: syn::Path {
                            leading_colon: Some(syn::Token![::](Span::call_site())),
                            segments: [
                                syn::PathSegment {
                                    ident: Ident::new("alkahest", Span::call_site()),
                                    arguments: syn::PathArguments::None,
                                },
                                syn::PathSegment {
                                    ident: Ident::new(bound, Span::call_site()),
                                    arguments: syn::PathArguments::None,
                                },
                            ]
                            .into_iter()
                            .collect(),
                        },
                    }))
                    .collect(),
                })
            })
            .collect(),
    })
}

impl FormulaArgs {
    pub fn from_derive(args: FormulaDeriveArgs) -> Self {
        FormulaArgs {
            _formula: Formula(Span::call_site()),
            where_clause: args.where_clause,
        }
    }

    pub fn add_default_where_clause(&mut self, generics: &syn::Generics) {
        if self.where_clause.is_none() {
            self.where_clause = impl_for_each_type_param(generics, "Element");
        }
    }
}

impl SerializeArgs {
    pub fn from_derive(args: SerializeDeriveArgs) -> Self {
        SerializeArgs {
            params: EasyMaybe::Nothing,
            _serialize: Serialize(Span::call_site()),
            _lt: syn::Token![<](Span::call_site()),
            formula: args.formula,
            _gt: syn::Token![>](Span::call_site()),
            variant: args.variant,
            where_clause: args.where_clause,
        }
    }

    pub fn merge_generics(&self, base: &syn::Generics) -> syn::Generics {
        merge_generics(base, &self.params, &self.where_clause)
    }
}

impl DeserializeArgs {
    pub fn from_derive(args: DeserializeDeriveArgs) -> Self {
        DeserializeArgs {
            params: EasyMaybe::Nothing,
            _deserialize: Deserialize(Span::call_site()),
            _lt: syn::Token![<](Span::call_site()),
            lifetime: EasyMaybe::Nothing,
            formula: args.formula,
            _gt: syn::Token![>](Span::call_site()),
            where_clause: args.where_clause,
        }
    }

    pub fn merge_generics(&self, base: &syn::Generics) -> syn::Generics {
        merge_generics(base, &self.params, &self.where_clause)
    }

    // If deserializer lifetime is not specified, 'de is used by default.
    pub fn deserializer_lifetime(&self) -> syn::Lifetime {
        if let EasyMaybe::Just(lifetime) = &self.lifetime {
            lifetime.lifetime.clone()
        } else {
            syn::Lifetime::new("'de", Span::call_site())
        }
    }

    // If deserializer lifetime is not specified, 'de is used by default and is added if missing.
    pub fn with_de_lifetime(&self, generics: &syn::Generics) -> syn::Generics {
        let mut generics = generics.clone();

        if let EasyMaybe::Nothing = &self.lifetime {
            let has_de_lifetime = generics.params.iter().any(|param| matches!(param, syn::GenericParam::Lifetime(lt) if lt.lifetime.ident == "de"));
            if !has_de_lifetime {
                generics
                    .params
                    .push(syn::GenericParam::Lifetime(syn::LifetimeParam::new(
                        syn::Lifetime::new("'de", Span::call_site()),
                    )));
            }
        }

        generics
    }
}

impl MixtureArgs {
    pub fn from_derive(args: MixtureDeriveArgs) -> Self {
        MixtureArgs {
            _mixture: Mixture(Span::call_site()),
            where_clause: args.where_clause,
        }
    }

    pub fn formula_args(&self, generics: &syn::Generics) -> FormulaArgs {
        // Only require `Element` for generics when implementing `Formula`.
        let where_clause = if self.where_clause.is_some() {
            self.where_clause.clone()
        } else {
            impl_for_each_type_param(generics, "Element")
        };

        FormulaArgs {
            _formula: Formula(Span::call_site()),
            where_clause: where_clause,
        }
    }

    pub fn serialize_deserialize_args(
        &self,
        generics: &syn::Generics,
    ) -> (SerializeArgs, DeserializeArgs) {
        let where_clause = if self.where_clause.is_some() {
            self.where_clause.clone()
        } else {
            impl_for_each_type_param(generics, "MixtureElement")
        };

        (
            SerializeArgs {
                params: EasyMaybe::Nothing,
                _serialize: Serialize(Span::call_site()),
                _lt: syn::Token![<](Span::call_site()),
                formula: syn::parse_quote!(Self),
                _gt: syn::Token![>](Span::call_site()),
                variant: proc_easy::EasyMaybe::Nothing,
                where_clause: where_clause.clone(),
            },
            DeserializeArgs {
                params: EasyMaybe::Nothing,
                _deserialize: Deserialize(Span::call_site()),
                _lt: syn::Token![<](Span::call_site()),
                lifetime: EasyMaybe::Nothing,
                formula: syn::parse_quote!(Self),
                _gt: syn::Token![>](Span::call_site()),
                where_clause: where_clause,
            },
        )
    }
}
