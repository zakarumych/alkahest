use syn::token::Comma;

proc_easy::easy_token!(path);
proc_easy::easy_token!(formula);

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

proc_easy::easy_parse! {
    pub struct SerializeArgs {
        pub formula: syn::Path,
        pub variant: proc_easy::EasyMaybe<Variant>,
        pub where_clause: Option<syn::WhereClause>,
    }
}
