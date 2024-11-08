mod rule_namespace;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(RuleNamespace, attributes(prefix))]
pub fn derive_rule_namespace(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    rule_namespace::derive_impl(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
