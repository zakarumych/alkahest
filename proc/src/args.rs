use proc_easy::{EasyMaybe, EasyPeek};
use proc_macro2::Span;
use syn::{GenericParam, Ident, WherePredicate, parse::Lookahead1, token::Comma};

proc_easy::easy_token!(path);
proc_easy::easy_token!(formula);
proc_easy::easy_token!(generate_module_file);
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
        pub generate_module_file: Option<generate_module_file>,
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

proc_easy::easy_token!(version);
proc_easy::easy_parse! {
    #[derive(Clone)]
    pub struct VersionArg {
        pub _version: version,
        pub _eq: syn::Token![=],
        pub value: syn::LitInt,
    }
}
proc_easy::easy_parse! {
    #[derive(Clone)]
    pub struct TrailingVersion {
        pub _comma: syn::Token![,],
        pub version: VersionArg,
    }
}
proc_easy::easy_token!(versions);

#[derive(Clone)]
pub struct VersionLinks {
    pub entries: syn::punctuated::Punctuated<VersionLink, syn::Token![,]>,
}

#[derive(Clone)]
pub struct VersionLink {
    pub version: syn::LitInt,
    pub formula: syn::Type,
}

impl syn::parse::Parse for VersionLink {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let version = input.parse()?;
        input.parse::<syn::Token![=]>()?;
        Ok(Self {
            version,
            formula: input.parse()?,
        })
    }
}

impl EasyPeek for VersionLinks {
    fn peek(lookahead: &Lookahead1) -> bool {
        lookahead.peek(syn::Token![,])
    }
    fn peek_stream(input: syn::parse::ParseStream) -> bool {
        input.peek(syn::Token![,])
    }
}

impl syn::parse::Parse for VersionLinks {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<syn::Token![,]>()?;
        input.parse::<versions>()?;
        let content;
        syn::parenthesized!(content in input);
        Ok(Self {
            entries: content.parse_terminated(VersionLink::parse, syn::Token![,])?,
        })
    }
}

pub fn revision_links(
    links: &EasyMaybe<VersionLinks>,
    current: u32,
) -> syn::Result<Vec<proc_macro2::TokenStream>> {
    let mut revisions = std::collections::BTreeSet::from([current]);
    let mut callbacks = Vec::new();
    if let EasyMaybe::Just(links) = links {
        for entry in &links.entries {
            let version: u32 = entry.version.base10_parse()?;
            if !revisions.insert(version) {
                return Err(syn::Error::new_spanned(
                    &entry.version,
                    "duplicate formula version, including the current version",
                ));
            }
            let formula = &entry.formula;
            callbacks.push(quote::quote!({
                assert!(<#formula as ::alkahest::Formula<#version>>::SCHEMA.version == #version);
                ::alkahest::schema::<#formula, #version>
            }));
        }
    }
    Ok(callbacks)
}

pub fn version_value(value: &EasyMaybe<TrailingVersion>) -> syn::Result<u32> {
    match value {
        EasyMaybe::Nothing => Ok(0),
        EasyMaybe::Just(value) => value.version.value.base10_parse(),
    }
}
pub fn field_version(attrs: &[syn::Attribute]) -> syn::Result<u32> {
    let mut version = None;
    for attr in attrs.iter().filter(|attr| attr.path().is_ident("alkahest")) {
        if version.is_some() {
            return Err(syn::Error::new_spanned(attr, "duplicate alkahest version"));
        }
        version = Some(attr.parse_args::<VersionArg>()?.value.base10_parse()?);
    }
    Ok(version.unwrap_or(0))
}
pub fn derive_args<T: syn::parse::Parse + Default>(attrs: &[syn::Attribute]) -> syn::Result<T> {
    let mut args = None;
    for attr in attrs.iter().filter(|attr| attr.path().is_ident("alkahest")) {
        if args.is_some() {
            return Err(syn::Error::new_spanned(
                attr,
                "duplicate alkahest arguments",
            ));
        }
        args = Some(attr.parse_args()?);
    }
    Ok(args.unwrap_or_default())
}
#[derive(Default)]
pub struct FormulaDeriveArgs {
    pub version: EasyMaybe<TrailingVersion>,
    pub versions: EasyMaybe<VersionLinks>,
    pub where_clause: Option<syn::WhereClause>,
}
impl syn::parse::Parse for FormulaDeriveArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let version = if input.peek(version) {
            EasyMaybe::Just(TrailingVersion {
                _comma: Default::default(),
                version: input.parse()?,
            })
        } else {
            EasyMaybe::Nothing
        };
        Ok(Self {
            version,
            versions: input.parse()?,
            where_clause: input.parse()?,
        })
    }
}

pub struct SerializeDeriveArgs {
    pub formula: syn::Path,
    pub variant: proc_easy::EasyMaybe<Variant>,
    pub version: EasyMaybe<TrailingVersion>,
    pub versions: EasyMaybe<VersionLinks>,
    pub where_clause: Option<syn::WhereClause>,
}
impl syn::parse::Parse for SerializeDeriveArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(version) {
            return Ok(Self {
                version: EasyMaybe::Just(TrailingVersion {
                    _comma: Default::default(),
                    version: input.parse()?,
                }),
                versions: input.parse()?,
                where_clause: input.parse()?,
                ..Self::default()
            });
        }
        Ok(Self {
            formula: input.parse()?,
            variant: input.parse()?,
            version: input.parse()?,
            versions: input.parse()?,
            where_clause: input.parse()?,
        })
    }
}

impl Default for SerializeDeriveArgs {
    fn default() -> Self {
        SerializeDeriveArgs {
            formula: syn::Path::from(syn::Ident::new("Self", Span::call_site())),
            variant: proc_easy::EasyMaybe::Nothing,
            version: EasyMaybe::Nothing,
            versions: EasyMaybe::Nothing,
            where_clause: None,
        }
    }
}

pub struct DeserializeDeriveArgs {
    pub formula: syn::Path,
    pub version: EasyMaybe<TrailingVersion>,
    pub versions: EasyMaybe<VersionLinks>,
    pub where_clause: Option<syn::WhereClause>,
}
impl syn::parse::Parse for DeserializeDeriveArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(version) {
            return Ok(Self {
                version: EasyMaybe::Just(TrailingVersion {
                    _comma: Default::default(),
                    version: input.parse()?,
                }),
                versions: input.parse()?,
                where_clause: input.parse()?,
                ..Self::default()
            });
        }
        Ok(Self {
            formula: input.parse()?,
            version: input.parse()?,
            versions: input.parse()?,
            where_clause: input.parse()?,
        })
    }
}

impl Default for DeserializeDeriveArgs {
    fn default() -> Self {
        DeserializeDeriveArgs {
            formula: syn::Path::from(syn::Ident::new("Self", Span::call_site())),
            version: EasyMaybe::Nothing,
            versions: EasyMaybe::Nothing,
            where_clause: None,
        }
    }
}

#[derive(Default)]
pub struct MixtureDeriveArgs {
    pub version: EasyMaybe<TrailingVersion>,
    pub versions: EasyMaybe<VersionLinks>,
    pub where_clause: Option<syn::WhereClause>,
}
impl syn::parse::Parse for MixtureDeriveArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let version = if input.peek(version) {
            EasyMaybe::Just(TrailingVersion {
                _comma: Default::default(),
                version: input.parse()?,
            })
        } else {
            EasyMaybe::Nothing
        };
        Ok(Self {
            version,
            versions: input.parse()?,
            where_clause: input.parse()?,
        })
    }
}

proc_easy::easy_parse! {
    pub struct FormulaArgs {
        pub _formula: Formula,
        pub version: EasyMaybe<TrailingVersion>,
    pub versions: EasyMaybe<VersionLinks>,
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
        pub version: EasyMaybe<TrailingVersion>,
    pub versions: EasyMaybe<VersionLinks>,
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
        pub version: EasyMaybe<TrailingVersion>,
    pub versions: EasyMaybe<VersionLinks>,
        pub where_clause: Option<syn::WhereClause>,
    }
}

proc_easy::easy_parse! {
    pub struct MixtureArgs {
        pub _mixture: Mixture,
        pub version: EasyMaybe<TrailingVersion>,
    pub versions: EasyMaybe<VersionLinks>,
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
                    attrs: Vec::new(),
                    lifetimes: None,
                    bounded_ty: syn::Type::Path(syn::TypePath {
                        attrs: Vec::new(),
                        qself: None,
                        path: syn::Path::from(param.ident.clone()),
                    }),
                    colon_token: syn::Token![:](Span::call_site()),
                    bounds: std::iter::once(syn::TypeParamBound::Trait(syn::TraitBound {
                        paren_token: None,
                        lifetimes: None,
                        modifiers: syn::TraitBoundModifiers::default(),
                        maybe: None,
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
            version: args.version,
            versions: args.versions,
            where_clause: args.where_clause,
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
            version: args.version,
            versions: args.versions,
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
            version: args.version,
            versions: args.versions,
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
            version: args.version,
            versions: args.versions,
            where_clause: args.where_clause,
        }
    }

    pub fn formula_args(&self, _generics: &syn::Generics) -> FormulaArgs {
        let where_clause = self.where_clause.clone();

        FormulaArgs {
            _formula: Formula(Span::call_site()),
            version: self.version.clone(),
            versions: self.versions.clone(),
            where_clause,
        }
    }

    pub fn serialize_deserialize_args(
        &self,
        generics: &syn::Generics,
    ) -> (SerializeArgs, DeserializeArgs) {
        let where_clause = if self.where_clause.is_some() {
            self.where_clause.clone()
        } else {
            impl_for_each_type_param(generics, "Component")
        };

        (
            SerializeArgs {
                params: EasyMaybe::Nothing,
                _serialize: Serialize(Span::call_site()),
                _lt: syn::Token![<](Span::call_site()),
                formula: syn::parse_quote!(Self),
                _gt: syn::Token![>](Span::call_site()),
                variant: proc_easy::EasyMaybe::Nothing,
                version: self.version.clone(),
                versions: self.versions.clone(),
                where_clause: where_clause.clone(),
            },
            DeserializeArgs {
                params: EasyMaybe::Nothing,
                _deserialize: Deserialize(Span::call_site()),
                _lt: syn::Token![<](Span::call_site()),
                lifetime: EasyMaybe::Nothing,
                formula: syn::parse_quote!(Self),
                _gt: syn::Token![>](Span::call_site()),
                version: self.version.clone(),
                versions: self.versions.clone(),
                where_clause,
            },
        )
    }
}

#[cfg(test)]
mod version_tests {
    use super::*;

    #[test]
    fn version_syntax_selects_same_root_for_every_derive() {
        let formula: FormulaDeriveArgs = syn::parse_quote!(version = 7);
        let serialize: SerializeDeriveArgs = syn::parse_quote!(version = 7);
        let deserialize: DeserializeDeriveArgs = syn::parse_quote!(version = 7);
        let mixture: MixtureDeriveArgs = syn::parse_quote!(version = 7);
        for version in [
            &formula.version,
            &serialize.version,
            &deserialize.version,
            &mixture.version,
        ] {
            assert_eq!(version_value(version).unwrap(), 7);
        }
        let serialize: SerializeDeriveArgs = syn::parse_quote!(Wire, version = 9);
        let deserialize: DeserializeDeriveArgs = syn::parse_quote!(Wire, version = 9);
        assert_eq!(version_value(&serialize.version).unwrap(), 9);
        assert_eq!(version_value(&deserialize.version).unwrap(), 9);
    }

    #[test]
    fn duplicate_and_invalid_versions_are_rejected() {
        assert!(syn::parse_str::<FormulaDeriveArgs>("version = 1, version = 2").is_err());
        assert!(syn::parse_str::<SerializeDeriveArgs>("Wire, version = 1, version = 2").is_err());
        let negative: FormulaDeriveArgs = syn::parse_quote!(version = -1);
        assert!(version_value(&negative.version).is_err());
        assert!(syn::parse_str::<FormulaDeriveArgs>("version = name").is_err());
        let overflow: FormulaDeriveArgs = syn::parse_quote!(version = 4294967296);
        assert!(version_value(&overflow.version).is_err());
        let input: syn::DeriveInput = syn::parse_quote! {
            #[alkahest(version = 1)]
            #[alkahest(version = 2)]
            struct Wire;
        };
        assert!(derive_args::<FormulaDeriveArgs>(&input.attrs).is_err());
    }

    #[test]
    fn version_links_reject_duplicates_invalid_numbers_and_malformed_entries() {
        let input: FormulaDeriveArgs = syn::parse_quote!(version = 2, versions(1 = Old, 3 = Next));
        assert_eq!(revision_links(&input.versions, 2).unwrap().len(), 2);
        let input: FormulaDeriveArgs = syn::parse_quote!(version = 2, versions(2 = Other));
        assert!(revision_links(&input.versions, 2).is_err());
        let input: FormulaDeriveArgs = syn::parse_quote!(version = 2, versions(1 = Old, 1 = Other));
        assert!(revision_links(&input.versions, 2).is_err());
        let input: FormulaDeriveArgs = syn::parse_quote!(version = 2, versions(4294967296 = Other));
        assert!(revision_links(&input.versions, 2).is_err());
        let input: FormulaDeriveArgs = syn::parse_quote!(version = 2, versions(-1 = Other));
        assert!(revision_links(&input.versions, 2).is_err());
        assert!(syn::parse_str::<FormulaDeriveArgs>("version = 2, versions(Old)").is_err());
        assert!(syn::parse_str::<FormulaDeriveArgs>("version = 2, versions(1 =)").is_err());
        assert!(
            syn::parse_str::<FormulaDeriveArgs>("version = 2, versions(), versions()").is_err()
        );
        let serialize: SerializeDeriveArgs = syn::parse_quote!(version = 2, versions(1 = Old));
        let deserialize: DeserializeDeriveArgs = syn::parse_quote!(version = 2, versions(1 = Old));
        assert_eq!(revision_links(&serialize.versions, 2).unwrap().len(), 1);
        assert_eq!(revision_links(&deserialize.versions, 2).unwrap().len(), 1);
    }

    #[test]
    fn separate_codecs_cannot_declare_formula_version_tables() {
        let input: syn::DeriveInput = syn::parse_quote!(
            struct Data;
        );
        let serialize: SerializeDeriveArgs =
            syn::parse_quote!(Wire, version = 2, versions(1 = Old));
        let deserialize: DeserializeDeriveArgs =
            syn::parse_quote!(Wire, version = 2, versions(1 = Old));
        assert!(
            crate::serialize::derive_impl(input.clone(), SerializeArgs::from_derive(serialize))
                .is_err()
        );
        assert!(
            crate::deserialize::derive_impl(input, DeserializeArgs::from_derive(deserialize))
                .is_err()
        );
    }

    #[test]
    fn misplaced_and_duplicate_child_versions_are_rejected() {
        let input: syn::DeriveInput = syn::parse_quote! {
            enum Wire { #[alkahest(version = 1)] Value(u8) }
        };
        assert!(crate::formula::validate_annotations(&input.data).is_err());
        let input: syn::DeriveInput = syn::parse_quote! {
            struct Wire(#[alkahest(version = 1)] #[alkahest(version = 2)] u8);
        };
        assert!(crate::formula::validate_annotations(&input.data).is_err());
        let input: syn::DeriveInput = syn::parse_quote! {
            struct Data(#[alkahest(version = 1)] u8);
        };
        assert!(crate::formula::reject_field_annotations(&input.data).is_err());
    }
}
