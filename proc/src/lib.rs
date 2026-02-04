use proc_macro::TokenStream;

use crate::args::TypeArgs;

extern crate proc_macro;

mod alk;
mod args;
mod deserialize;
mod formula;
mod include;
mod mixture;
mod module;
mod serialize;

/// Derives `Formula` implementation for the annotated type.
#[proc_macro_derive(Formula, attributes(alkahest))]
pub fn formula(item: TokenStream) -> TokenStream {
    formula::derive(item)
}

/// Derives `Serialize` implementation for the annotated type.
/// Attribute `#[alkahest(FormulaType))]` specifies to implement `Serialize<FormulaType>`.
/// Otherwise implements `Serialize<Self>`.
#[proc_macro_derive(Serialize, attributes(alkahest))]
pub fn serialize(item: TokenStream) -> TokenStream {
    serialize::derive(item)
}

/// Derives `Deserialize` implementation for the annotated type.
/// Attribute `#[alkahest(FormulaType))]` specifies to implement `Deserialize<FormulaType>`.
/// Otherwise implements `Deserialize<Self>`.
#[proc_macro_derive(Deserialize, attributes(alkahest))]
pub fn deserialize(item: TokenStream) -> TokenStream {
    deserialize::derive(item)
}

/// Derives `Mixture` implementation for the annotated type.
/// Mixture is a combination of `Formula`, `Serialize<Self>` and `Deserialize<Self>`.
#[proc_macro_derive(Mixture, attributes(alkahest))]
pub fn mixture(item: TokenStream) -> TokenStream {
    mixture::derive(item)
}

/// Includes formulas from a `.alk` file as specified in the input token stream.
#[proc_macro]
pub fn include_formulas(item: TokenStream) -> TokenStream {
    let path = syn::parse_macro_input!(item as syn::LitStr);
    include::include_formulas(path)
}

/// Proc-attribute with richer syntax than derive macros.
///
/// When applied to a module, expands to the generated module with formulas from a `.alk` file.
/// Usage:
///   `#[alkahest(path = "path/to/module.alk")] mod my_module {}`
///   `#[alkahest] mod my_module {}` // uses `my_module.alk`
///
/// When applied to a type definition, works like derive macro.
/// The difference is how arguments to derive macros are specified.
///
/// `#[alkahest(Formula)]` is equivalent to `#[derive(Formula)]`
/// `#[alkahest(Serialize<FormulaType>)]` is equivalent to `#[derive(Serialize)]#[alkahest(FormulaType)]`
/// `#[alkahest(Deserialize<FormulaType>)]` is equivalent to `#[derive(Deserialize)]#[alkahest<FormulaType>]`
/// `#[alkahest(Mixture)]` is equivalent to `#[derive(Mixture)]`
///
/// Any of `Formula`, `Serialize`, `Deserialize` and `Mixture` can be followed by where clause.
///
/// `#[alkahest(Formula where A: Formula)]`
///
/// `Serialize` and `Deserialize` can be prefixed with `for` to specify additional generic parameters not present on the type.
///
/// ```
/// #[alkahest(Formula)]
/// struct GenericFormula<T> {}
///
/// #[alkahest(for<A: Formula> Serialize<GenericFormula<A>> where T: Serialize<A>)]
/// struct GenericStruct<T> {}
/// ```
#[proc_macro_attribute]
pub fn alkahest(attr: TokenStream, item: TokenStream) -> TokenStream {
    if let Ok(item) = syn::parse::<module::ModuleItem>(item.clone()) {
        return module::alkahest(attr, item);
    }

    if let Ok(input) = syn::parse::<syn::DeriveInput>(item.clone()) {
        return type_alkahest(attr, input);
    }

    syn::Error::new_spanned(
        proc_macro2::TokenStream::from(item),
        "the #[alkahest] attribute can only be applied to modules or type definitions",
    )
    .to_compile_error()
    .into()
}

fn type_alkahest(attr: TokenStream, input: syn::DeriveInput) -> TokenStream {
    match type_alkahest_impl(attr, input) {
        Ok(tokens) => tokens.into(),
        Err(err) => proc_macro::TokenStream::from(err.to_compile_error()),
    }
}

fn type_alkahest_impl(
    attr: TokenStream,
    input: syn::DeriveInput,
) -> syn::Result<proc_macro2::TokenStream> {
    let args = syn::parse::<args::TypeArgs>(attr)?;

    let mut tokens = quote::quote! { #input };

    match args {
        TypeArgs::Formula(args) => tokens.extend(formula::derive_impl(input, args)?),
        TypeArgs::Serialize(args) => tokens.extend(serialize::derive_impl(input, args)?),
        TypeArgs::Deserialize(args) => tokens.extend(deserialize::derive_impl(input, args)?),
        TypeArgs::Mixture(args) => tokens.extend(mixture::derive_impl(input, args)?),
    }

    Ok(tokens)
}
