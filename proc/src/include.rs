use crate::alk::{module_from_path, module_to_tokens};

pub fn include_formulas(path: syn::LitStr) -> proc_macro::TokenStream {
    match include_formulas_impl(path) {
        Ok(tokens) => tokens.into(),
        Err(err) => proc_macro::TokenStream::from(err.to_compile_error()),
    }
}

fn include_formulas_impl(path: syn::LitStr) -> syn::Result<proc_macro2::TokenStream> {
    let module_path = path.value();
    let error_span = path.span();

    let module = module_from_path(module_path.as_ref(), error_span)?;

    let mut tokens = proc_macro2::TokenStream::new();

    tokens.extend(quote::quote! {
        // This forces proc-macro to recompile when the module file changes
        const MODULE_SOURCE: &'static str = include_str!(#module_path);
    });

    module_to_tokens(&module, &mut tokens);

    Ok(tokens)
}
