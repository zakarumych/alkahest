use proc_easy::{EasyBraced, EasyMaybe};
use quote::ToTokens;

use crate::{
    alk::{module_from_path, module_to_tokens},
    args::ModuleArgs,
};

proc_easy::easy_parse! {
    struct Empty;
}

proc_easy::easy_parse! {
    pub struct ModuleItem {
        mod_token: syn::Token![mod],
        ident: syn::Ident,
        content: EasyMaybe<EasyBraced<Empty>>,
        semi: EasyMaybe<syn::Token![;]>,
    }
}

pub(crate) fn alkahest(attr: proc_macro::TokenStream, item: ModuleItem) -> proc_macro::TokenStream {
    match alkahest_impl(attr, item) {
        Ok(tokens) => tokens.into(),
        Err(err) => proc_macro::TokenStream::from(err.to_compile_error()),
    }
}

fn alkahest_impl(
    attr: proc_macro::TokenStream,
    item: ModuleItem,
) -> syn::Result<proc_macro2::TokenStream> {
    let args = syn::parse::<ModuleArgs>(attr)?;

    let module_path = match &args.path {
        None => item.ident.to_string() + ".alk",
        Some(path) => path.module.value(),
    };

    let error_span = match &args.path {
        None => item.ident.span(),
        Some(path) => path.module.span(),
    };

    let module = module_from_path(module_path.as_ref(), error_span)?;

    let mut tokens = proc_macro2::TokenStream::new();

    item.mod_token.to_tokens(&mut tokens);
    item.ident.to_tokens(&mut tokens);

    syn::token::Brace::default().surround(&mut tokens, |tokens| {
        tokens.extend(quote::quote! {
            // This forces proc-macro to recompile when the module file changes
            const MODULE_SOURCE: &'static str = include_str!(#module_path);
        });

        module_to_tokens(&module, tokens);
    });

    Ok(tokens)
}
