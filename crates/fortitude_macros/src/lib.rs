use crate::violation_metadata::violation_metadata;

mod has_node;
mod map_codes;
mod rule_code_prefix;
mod rule_namespace;
mod violation_metadata;

use proc_macro::TokenStream;
use syn::{DeriveInput, Error, ItemFn, parse_macro_input};

#[proc_macro_derive(ViolationMetadata, attributes(violation_metadata))]
pub fn derive_violation_metadata(item: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(item);

    violation_metadata(input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

#[proc_macro_derive(RuleNamespace, attributes(prefix))]
pub fn derive_rule_namespace(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    rule_namespace::derive_impl(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_attribute]
pub fn map_codes(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn);
    map_codes::map_codes(&func)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(HasNode)]
pub fn derive_has_node(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    has_node::derive_impl(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
