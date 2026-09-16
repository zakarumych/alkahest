use std::{fs, path::PathBuf};

use alkahest_parse::Module;
use proc_macro2::Span;
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
        vis: syn::Visibility,
        mod_token: syn::Token![mod],
        ident: syn::Ident,
        semi: syn::Token![;],
    }
}

pub(crate) fn alkahest(attr: proc_macro::TokenStream, item: ModuleItem) -> proc_macro::TokenStream {
    match alkahest_impl(attr, item) {
        Ok(tokens) => tokens.into(),
        Err(err) => proc_macro::TokenStream::from(err.to_compile_error()),
    }
}

fn generate_source(module: &Module) -> String {
    let mut tokens = proc_macro2::TokenStream::new();
    module_to_tokens(module, &mut tokens);
    tokens.to_string()
}

fn resolve_relative_path(mut path: &std::path::Path, span: Span) -> Result<PathBuf, syn::Error> {
    if path.is_absolute() {
        return Ok(path.to_owned());
    }

    let mut base_path = match path.strip_prefix("$") {
        Ok(suffix) => {
            path = suffix;

            let base_path = std::env::var_os("CARGO_MANIFEST_DIR")
                .ok_or_else(|| syn::Error::new(span, "Cannot determine package root directory"))?;

            PathBuf::from(base_path)
        }
        Err(_) => {
            let mut source_path = span
            .local_file()
            .ok_or_else(|| syn::Error::new(span, "Cannot determine the path of the source file. Use root relative path or absolute path for .alk file"))?;

            if !source_path.pop() {
                return Err(syn::Error::new(
                    span,
                    format!(
                        "Cannot determine the directory of the source file {}",
                        source_path.display()
                    ),
                ));
            }

            source_path
        }
    };

    if !base_path.is_absolute() {
        let mut cd = std::env::current_dir().map_err(|err| {
            syn::Error::new(span, format!("std::env::current_dir() error: {}", err))
        })?;

        cd.push(base_path);
        base_path = cd;
    }

    base_path.push(path);

    Ok(base_path)
}

fn alkahest_impl(
    attr: proc_macro::TokenStream,
    module_item: ModuleItem,
) -> syn::Result<proc_macro2::TokenStream> {
    let args = syn::parse::<ModuleArgs>(attr)?;

    let module_name = module_item.ident.to_string();

    let module_path = match &args.path {
        None => format!("{module_name}.alk"),
        Some(path) => path.module.value(),
    };

    let error_span = match &args.path {
        None => module_item.ident.span(),
        Some(path) => path.module.span(),
    };

    let resolved_path = resolve_relative_path(module_path.as_ref(), error_span)?;

    let resolved_path = resolved_path.to_str().ok_or_else(|| {
        syn::Error::new(
            error_span,
            format!(
                "Failed to convert path {} to UTF-8",
                resolved_path.display()
            ),
        )
    })?;

    let module = module_from_path(resolved_path.as_ref(), error_span)?;

    let mut tokens = proc_macro2::TokenStream::new();
    module_item.vis.to_tokens(&mut tokens);
    module_item.mod_token.to_tokens(&mut tokens);
    module_item.ident.to_tokens(&mut tokens);

    if args.generate_module_file.is_none() {
        syn::token::Brace::default().surround(&mut tokens, |tokens| {
            tokens.extend(quote::quote! {
                // This forces proc-macro to re-run when the module file changes
                const MODULE_SOURCE: &'static str = include_str!(#resolved_path);
            });

            module_to_tokens(&module, tokens);
        });

        Ok(tokens)
    } else {
        module_item.semi.to_tokens(&mut tokens);

        let source = generate_source(&module);

        let generated_path = std::path::Path::new(&resolved_path).with_added_extension("rs");

        fs::write(&generated_path, source).map_err(|err| {
            syn::Error::new_spanned(
                &module_item.ident,
                format!("Failed to write generated alkahest module source: {err}"),
            )
        })?;

        let source_path = generated_path.to_str().ok_or_else(|| {
            syn::Error::new_spanned(
                &module_item.ident,
                "Generated alkahest module path is not UTF-8",
            )
        })?;

        Ok(quote::quote! {
            // This forces proc-macro to re-run when the module file changes
            const _: &'static str = include_str!(#resolved_path);
            #[path = #source_path]
            #tokens
        })
    }
}
