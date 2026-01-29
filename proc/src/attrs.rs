use syn::token::Comma;

proc_easy::easy_token!(path);

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
