proc_easy::easy_token!(Formula);
proc_easy::easy_token!(Serialize);
proc_easy::easy_token!(SerializeRef);
proc_easy::easy_token!(Deserialize);

proc_easy::easy_parse! {
    struct Params {
        token: syn::Token![for],
        generics: syn::Generics,
    }
}

proc_easy::easy_parse! {
    struct Variant {
        at: syn::Token![@],
        ident: syn::Ident,
    }
}

proc_easy::easy_parse! {
    struct SerializeParams {
        lt_token: syn::Token![<],
        formula: syn::Path,
        variant: proc_easy::EasyMaybe<Variant>,
        gt_token: syn::Token![>],
    }
}

proc_easy::easy_parse! {
    struct DeserializeParams {
        lt_token: syn::Token![<],
        lifetime: syn::Lifetime,
        comma_token: syn::Token![,],
        formula: syn::Path,
        gt_token: syn::Token![>],
    }
}

proc_easy::easy_parse! {
    enum ImplTrait {
        Formula(Formula),
        Serialize(Serialize, proc_easy::EasyMaybe<SerializeParams>),
        SerializeRef(SerializeRef, proc_easy::EasyMaybe<SerializeParams>),
        Deserialize(Deserialize, proc_easy::EasyMaybe<DeserializeParams>),
    }
}

proc_easy::easy_parse! {
    struct ImplBlock {
        params: proc_easy::EasyMaybe<Params>,
        impl_trait: ImplTrait,
        where_clause: Option<syn::WhereClause>,
    }
}

struct ImplBlocks {
    blocks: syn::punctuated::Punctuated<ImplBlock, syn::Token![,]>,
}

impl syn::parse::Parse for ImplBlocks {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(ImplBlocks {
            blocks: syn::punctuated::Punctuated::parse_separated_nonempty(input)?,
        })
    }
}

impl ImplBlock {
    fn split(self) -> (ImplTrait, Option<syn::Generics>) {
        let mut generics = match self.params {
            proc_easy::EasyMaybe::Nothing => None,
            proc_easy::EasyMaybe::Just(params) => Some(params.generics),
        };

        if let Some(where_clause) = self.where_clause {
            generics
                .get_or_insert_with(syn::Generics::default)
                .make_where_clause()
                .predicates
                .extend(where_clause.predicates);
        }

        (self.impl_trait, generics)
    }
}

pub struct FormulaArgs {
    pub generics: Option<syn::Generics>,
}

impl FormulaArgs {
    pub fn empty() -> Self {
        FormulaArgs { generics: None }
    }
}

pub struct SerializeArgs {
    pub formula: Option<syn::Path>,
    pub generics: Option<syn::Generics>,
    pub variant: Option<syn::Ident>,
}

impl SerializeArgs {
    pub fn empty() -> Self {
        SerializeArgs {
            formula: None,
            generics: None,
            variant: None,
        }
    }
}

pub struct DeserializeArgs {
    pub formula: Option<syn::Path>,
    pub generics: Option<syn::Generics>,
    pub lifetime: Option<syn::Lifetime>,
}

impl DeserializeArgs {
    pub fn empty() -> Self {
        DeserializeArgs {
            formula: None,
            generics: None,
            lifetime: None,
        }
    }
}

pub struct Args {
    pub formula: Option<FormulaArgs>,
    pub serialize: Option<SerializeArgs>,
    pub serialize_ref: Option<SerializeArgs>,
    pub deserialize: Option<DeserializeArgs>,
}

impl Args {
    pub fn parse_attributes(attrs: proc_macro2::TokenStream) -> syn::Result<Self> {
        let blocks: ImplBlocks = syn::parse2(attrs)?;

        let mut formula: Option<FormulaArgs> = None;
        let mut serialize: Option<SerializeArgs> = None;
        let mut serialize_ref: Option<SerializeArgs> = None;
        let mut deserialize: Option<DeserializeArgs> = None;

        for block in blocks.blocks {
            let (impl_trait, generics) = block.split();
            match impl_trait {
                ImplTrait::Formula(_) => formula = Some(FormulaArgs { generics }),
                ImplTrait::Serialize(_, params) => {
                    let (formula, variant) = match params {
                        proc_easy::EasyMaybe::Just(params) => (
                            Some(path_make_expr_style(params.formula)),
                            match params.variant {
                                proc_easy::EasyMaybe::Just(variant) => Some(variant.ident),
                                proc_easy::EasyMaybe::Nothing => None,
                            },
                        ),
                        proc_easy::EasyMaybe::Nothing => (None, None),
                    };

                    serialize = Some(SerializeArgs {
                        formula,
                        generics,
                        variant,
                    });
                }
                ImplTrait::SerializeRef(_, params) => {
                    let (formula, variant) = match params {
                        proc_easy::EasyMaybe::Just(params) => (
                            Some(path_make_expr_style(params.formula)),
                            match params.variant {
                                proc_easy::EasyMaybe::Just(variant) => Some(variant.ident),
                                proc_easy::EasyMaybe::Nothing => None,
                            },
                        ),
                        proc_easy::EasyMaybe::Nothing => (None, None),
                    };

                    serialize_ref = Some(SerializeArgs {
                        formula,
                        generics,
                        variant,
                    });
                }
                ImplTrait::Deserialize(_, params) => {
                    let (formula, lifetime) = match params {
                        proc_easy::EasyMaybe::Just(params) => (
                            Some(path_make_expr_style(params.formula)),
                            Some(params.lifetime),
                        ),
                        proc_easy::EasyMaybe::Nothing => (None, None),
                    };

                    deserialize = Some(DeserializeArgs {
                        formula,
                        generics,
                        lifetime,
                    });
                }
            }
        }

        Ok(Args {
            formula,
            serialize,
            serialize_ref,
            deserialize,
        })
    }
}

pub fn path_make_expr_style(mut path: syn::Path) -> syn::Path {
    for seg in &mut path.segments {
        if let syn::PathArguments::AngleBracketed(ref mut args) = seg.arguments {
            args.colon2_token = Some(<syn::Token![::]>::default());
        }
    }
    path
}
