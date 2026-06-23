use crate::violation_metadata::violation_metadata;

mod has_name;
mod has_node;
mod map_codes;
mod rule_code_prefix;
mod rule_namespace;
mod violation_metadata;

use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::{DeriveInput, Error, ItemFn, LitStr, parse_macro_input};
use tree_sitter::Language;

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

#[proc_macro_derive(HasName)]
pub fn derive_has_name(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    has_name::derive_impl(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn id_for_node_kind(token_stream: TokenStream, named: bool) -> TokenStream {
    let string_literal: LitStr = parse_macro_input!(token_stream);

    // Get the string value and calculate its length
    let requested_kind = string_literal.value();

    let language: Language = tree_sitter_fortran::LANGUAGE.into();
    let found_id = language.id_for_node_kind(&requested_kind, named);

    if found_id != 0 {
        quote! {
            #found_id
        }
    } else {
        quote_spanned!(
            string_literal.span() =>
            compile_error!("This is not a valid node kind in the tree-sitter-fortran grammar")
        )
    }
    .into()
}

/// Given a string literal of a named node, return the corresponding tree-sitter
/// kind_id
#[proc_macro]
pub fn kind(token_stream: TokenStream) -> TokenStream {
    id_for_node_kind(token_stream, true)
}

/// Given a string literal of an unnamed node (that is, a keyword), return the
/// corresponding tree-sitter kind_id
#[proc_macro]
pub fn kw(token_stream: TokenStream) -> TokenStream {
    id_for_node_kind(token_stream, false)
}

/// Given a string literal of a field name, return the corresponding tree-sitter
/// field_id
#[proc_macro]
pub fn field(token_stream: TokenStream) -> TokenStream {
    let string_literal: LitStr = parse_macro_input!(token_stream);

    // Get the string value and calculate its length
    let requested_keyword = string_literal.value();

    let language: Language = tree_sitter_fortran::LANGUAGE.into();
    let found_id = language.field_id_for_name(&requested_keyword);

    if let Some(found_id) = found_id {
        let id_number: u16 = found_id.into();
        quote! {
            std::num::NonZeroU16::new(#id_number).unwrap()
        }
    } else {
        quote_spanned!(
            string_literal.span() =>
            compile_error!("This is not a valid field in the tree-sitter-fortran grammar")
        )
    }
    .into()
}
