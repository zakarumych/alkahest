extern crate proc_macro;

mod attrs;

use attrs::ModuleArgs;
use proc_easy::EasyBraced;
use proc_macro::{Span, TokenStream};
use quote::ToTokens;

proc_easy::easy_parse! {
    struct Empty;
}

proc_easy::easy_parse! {
    struct Module {
        mod_token: syn::Token![mod],
        ident: syn::Ident,
        semi: EasyBraced<Empty>
    }
}

#[proc_macro_attribute]
pub fn alkahest(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as Module);

    match alkahest_impl(attr, input) {
        Ok(tokens) => tokens.into(),
        Err(err) => TokenStream::from(err.to_compile_error()),
    }
}

fn alkahest_impl(attr: TokenStream, input: Module) -> syn::Result<proc_macro2::TokenStream> {
    let args = syn::parse::<ModuleArgs>(attr)?;

    let module_relative_path = match &args.path {
        None => input.ident.to_string() + ".alk",
        Some(path) => path.module.value(),
    };

    let source_path = Span::call_site().local_file().ok_or_else(|| {
        syn::Error::new(
            input.ident.span(),
            "Cannot determine the path of the source file",
        )
    })?;

    let base_path = source_path.parent().ok_or_else(|| {
        syn::Error::new(
            input.ident.span(),
            "Cannot determine the directory of the source file",
        )
    })?;

    let module_path = base_path.join(module_relative_path);

    let module_source = std::fs::read_to_string(&*module_path).map_err(|err| {
        syn::Error::new(
            input.ident.span(),
            format!("Failed to read module file: {}", err),
        )
    })?;

    let module = alkahest_parse::parse_module(module_source).map_err(|err| {
        syn::Error::new(
            input.ident.span(),
            format!("Failed to parse module file: {}", err),
        )
    })?;

    let mut output = proc_macro2::TokenStream::new();

    input.mod_token.to_tokens(&mut output);
    input.ident.to_tokens(&mut output);

    syn::token::Brace::default().surround(&mut output, |tokens| {
        tokens.extend(quote::quote! {
            use alkahest_core::private::*;
        });

        alkahest_rust_gen::module_to_tokens(&module, tokens);
    });

    Ok(output)
}
