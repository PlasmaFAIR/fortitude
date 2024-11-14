// Adapted from from ruff
// Copyright 2022 Charles Marsh
// SPDX-License-Identifier: MIT

use std::cmp::Reverse;
use std::collections::HashSet;

use quote::quote;
use syn::spanned::Spanned;
use syn::{Data, DataEnum, DeriveInput, Error, ExprLit, Lit, Meta, MetaNameValue};

pub(crate) fn derive_impl(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let DeriveInput {
        ident,
        data: Data::Enum(DataEnum { variants, .. }),
        ..
    } = input
    else {
        return Err(Error::new(
            input.ident.span(),
            "only named fields are supported",
        ));
    };

    let mut parsed = Vec::new();

    let mut common_prefix_match_arms = quote!();
    let mut name_match_arms = quote!();
    let mut description_match_arms = quote!();

    let mut all_prefixes = HashSet::new();

    for variant in variants {
        let mut first_chars = HashSet::new();
        let prefixes: Result<Vec<_>, _> = variant
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("prefix"))
            .map(|attr| {
                let Meta::NameValue(MetaNameValue{value: syn::Expr::Lit (ExprLit { lit: Lit::Str(lit), ..}), ..}) = &attr.meta else {
                    return Err(Error::new(attr.span(), r#"expected attribute to be in the form of [#prefix = "..."]"#));
                };
                let str = lit.value();
                match str.chars().next() {
                    None => return Err(Error::new(lit.span(), "expected prefix string to be non-empty")),
                    Some(c) => if !first_chars.insert(c) {
                        return Err(Error::new(lit.span(), format!("this variant already has another prefix starting with the character '{c}'")))
                    }
                }
                if !all_prefixes.insert(str.clone()) {
                    return Err(Error::new(lit.span(), "prefix has already been defined before"));
                }
                Ok(str)
            })
            .collect();
        let prefixes = prefixes?;

        if prefixes.is_empty() {
            return Err(Error::new(
                variant.span(),
                r#"Missing #[prefix = "..."] attribute"#,
            ));
        }

        let Some(doc_attr) = variant
            .attrs
            .iter()
            .find(|attr| attr.path().is_ident("doc"))
        else {
            return Err(Error::new(variant.span(), "expected a doc comment"));
        };

        let variant_ident = variant.ident;

        description_match_arms.extend(quote! { Self::#variant_ident => stringify!(#doc_attr),});
        name_match_arms.extend(quote! {Self::#variant_ident => stringify!(#variant_ident),});

        for lit in &prefixes {
            parsed.push((lit.clone(), variant_ident.clone()));
        }

        if let [prefix] = &prefixes[..] {
            common_prefix_match_arms.extend(quote! { Self::#variant_ident => #prefix, });
        } else {
            // There is more than one prefix. We already previously asserted
            // that prefixes of the same variant don't start with the same character
            // so the common prefix for this variant is the empty string.
            common_prefix_match_arms.extend(quote! { Self::#variant_ident => "", });
        }
    }

    parsed.sort_by_key(|(prefix, ..)| Reverse(prefix.len()));

    let mut if_statements = quote!();

    for (prefix, field) in parsed {
        let ret_str = quote!(rest);
        if_statements.extend(quote! {
            if let Some(rest) = code.strip_prefix(#prefix) {
                return Some((#ident::#field, #ret_str));
            }
        });
    }

    Ok(quote! {
        #[automatically_derived]
        impl crate::registry::RuleNamespace for #ident {
            fn parse_code(code: &str) -> Option<(Self, &str)> {
                if let Ok(variant) = Self::from_str(code) {
                    return Some((variant, ""));
                }
                #if_statements
                None
            }

            fn common_prefix(&self) -> &'static str {
                match self { #common_prefix_match_arms }
            }

            fn name(&self) -> &'static str {
                match self { #name_match_arms }
            }

            fn description(&self) -> &'static str {
                match self { #description_match_arms }
            }
        }
    })
}
