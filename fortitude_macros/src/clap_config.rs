// Adapted from clap_config
// https://github.com/gibfahn/clap_config
// Copyright 2024 Gibson Fahnestock
// SPDX-License-Identifier: MIT

use heck::{ToKebabCase, ToSnakeCase};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Comma;
use syn::{
    AngleBracketedGenericArguments, Data, DeriveInput, Expr, ExprPath, Field, Fields,
    GenericArgument, Ident, Meta, PathArguments, PathSegment, Token, Type, TypePath, TypeTuple,
    Variant,
};

const CLAP_CONFIG_ATTR_NAME: &str = "clap_config";

pub(crate) fn derive_impl(input: DeriveInput) -> proc_macro::TokenStream {
    // Name of the struct we're creating a Config version of.
    let input_ident = input.ident;
    // Name of the config struct we' creating.
    let config_ident = &get_config_ident(&input_ident);

    let config_fields;
    let merge_method;

    let data = &input.data;
    match *data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => {
                let input_fields = &fields.named;
                config_fields = make_fields_optional(input_fields);
                merge_method = struct_merge_method(config_ident, input_fields);
            }
            _ => unimplemented!("Unimplemented struct field"),
        },
        Data::Enum(ref data) => {
            let variants = &data.variants;
            config_fields = variants_to_fields(variants);
            merge_method = enum_merge_method(config_ident, variants);
        }
        _ => unimplemented!("Unimplemented input type"),
    }

    let output = quote!(
        // We currently default everything to pub by default, so don't warn about it.
        #[allow(private_interfaces)]
        #[derive(
            std::default::Default,
            std::fmt::Debug,
            std::clone::Clone,
            serde::Deserialize,
            serde::Serialize,
        )]
        #[serde(rename_all = "kebab-case", deny_unknown_fields)]
        pub struct #config_ident {
            #config_fields
        }

        impl #input_ident {
            #merge_method
        }
    );
    proc_macro::TokenStream::from(output)
}

fn variants_to_fields(variants: &Punctuated<syn::Variant, Comma>) -> TokenStream {
    let optional_fields = variants.iter().filter_map(|v| {
        let name = Ident::new(
            &v.ident.to_string().as_str().to_snake_case(),
            v.ident.span(),
        );
        // Skip unit subcommand fields (as they have no opts to configure).
        let f = get_variant_field(v)?;
        let ty = make_subcommand_ty(&f.ty);
        Some(quote_spanned!(f.span()=> pub #name: std::option::Option<#ty>))
    });

    quote! {
        #(
            #[serde(skip_serializing_if = "Option::is_none")]
            #optional_fields
        ),*
    }
}

/**
Get the field to use for a variant of a subcommand if there is an associated fieeld.

e.g. for `SubCommand::SubCommandA(opts: Opts)` -> `Some(opts: Opts)`
e.g. for `SubCommand::SubCommandA(Opts)` -> `Some(Opts)`
e.g. for `SubCommand::SubCommandA` -> `None`
*/
fn get_variant_field(v: &Variant) -> Option<&Field> {
    match v.fields {
        Fields::Named(ref fields) => Some(
            fields
                .named
                .iter()
                .next()
                .expect("Expected enum variant to have a single named field"),
        ),
        Fields::Unnamed(ref fields) => Some(
            fields
                .unnamed
                .iter()
                .next()
                .expect("Expected enum variant to have a single unnamed field"),
        ),
        Fields::Unit => None,
    }
}

/// Convert any fields that aren't already `Option<...>` to `Option<...>` fields, ensuring
/// everything is optional.
fn make_fields_optional(fields: &Punctuated<Field, Comma>) -> TokenStream {
    let mut optional_fields = vec![];

    for f in fields {
        let name = &f.ident;
        let f_ty = &f.ty;
        let mut ty = quote!(#f_ty);

        match is_field_marked_skipped(f) {
            Ok(true) => continue,
            Ok(false) => (),
            Err(e) => return quote!(#e),
        }

        if is_vec_tuple_string(f) {
            ty = quote!(
                std::collections::BTreeMap<std::string::String, std::string::String>
            );
        }
        if is_subcommand_field(f).expect("Failed to check if subcommand field is field") {
            let ty = make_subcommand_ty(strip_optional_wrapper_if_present(f).unwrap_or(&f.ty));
            optional_fields.push(quote_spanned!(f.span()=>
                #[serde(flatten)]
                pub #name: std::option::Option<#ty>
            ))
        } else if strip_optional_wrapper_if_present(f).is_some() {
            optional_fields.push(quote_spanned!(f.span()=> pub #name: #ty))
        } else {
            optional_fields.push(quote_spanned!(f.span()=> pub #name: std::option::Option<#ty>))
        }
    }

    quote! {
        #(
            #[serde(skip_serializing_if = "Option::is_none")]
            #optional_fields
        ),*
    }
}

fn make_subcommand_ty(ty: &Type) -> Type {
    if let Type::Path(path) = ty {
        let ident = path
            .path
            .require_ident()
            .expect("Expected subcommand type to be bare identifier.");
        let new_ident = get_config_ident(ident);
        Type::Path(TypePath {
            qself: None,
            path: syn::Path::from(PathSegment::from(new_ident)),
        })
    } else {
        panic!("Expected the subcommand type to be a bare identifier type.");
    }
}

fn get_config_ident(ident: &Ident) -> Ident {
    format_ident!("{ident}Config")
}

/**
Generate method that merges our config into the clap-generated struct, with precedence being:

- Things specified via `--arg` or `$ENV_VAR`
- Things in the config
- Clap defaults
*/
fn struct_merge_method(config_ident: &Ident, fields: &Punctuated<Field, Comma>) -> TokenStream {
    let struct_fields = fields.iter().map(|f| {
        let name = &f.ident;
        quote!(#name)
    });

    let field_updates = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        let span = ty.span();
        let name_str = name.as_ref().map(|name| name.to_string()).expect("Expected field to have a name");
        let name_str = name_str.strip_prefix("r#").unwrap_or(&name_str);

        let is_skipped = match is_field_marked_skipped(f) {
            Ok(b) => b,
            Err(e) => return quote!(#e),
        };

        let config_value_expr = if is_skipped {
            quote!(None)
        } else {
            quote!(config.as_mut().and_then(|c| c.#name.take()))
        };

        if is_subcommand_field(f).expect("Failed to check if field is subcommand.") {
            if let Some(stripped_ty) = strip_optional_wrapper_if_present(f) {
                quote_spanned! {span=>
                    let #name: #ty = {
                        if let Some((subcommand_name,
                                     subcommand_matches)) = matches.remove_subcommand() {
                            Some(#stripped_ty :: from_merged(
                                subcommand_name,
                                subcommand_matches,
                                config.as_ref().and_then(|c| c.#name.clone())
                            ))
                        } else {
                            None
                        }
                    };
                }
            } else {
                quote_spanned! {span=>
                    let (subcommand_name, subcommand_matches) = matches.remove_subcommand().expect("Subcommand is required, so expected it to be set.");
                    let #name: #ty = #ty :: from_merged(
                        subcommand_name,
                        subcommand_matches,
                        config.as_ref().and_then(|c| c.#name.clone())
                    );
                }
            }
        } else if let Some(stripped_ty) = strip_optional_wrapper_if_present(f) {
            // User-specified field's type was `Option<T>`
            quote_spanned! {span=>
                let #name: #ty = {
                    let config_value: #ty = #config_value_expr;
                    if matches.contains_id(#name_str) {
                        let value_source = matches.value_source(#name_str).expect("checked contains_id");
                        let matches_value: #stripped_ty = matches.remove_one(#name_str).expect("checked contains_id");
                        if value_source == clap::parser::ValueSource::DefaultValue {
                            Some(config_value.unwrap_or(matches_value))
                        } else {
                            Some(matches_value)
                        }
                    } else {
                        config_value
                    }
                };
            }
        } else if is_vec_tuple_string(f) {
            quote_spanned! {span=>
                let #name: #ty = {
                    let config_value: std::option::Option<std::collections::BTreeMap<std::string::String, std::string::String>> = #config_value_expr;
                    if matches.contains_id(#name_str) {
                        let value_source = matches.value_source(#name_str).expect("checked contains_id");
                        let matches_value: #ty = matches.remove_many(#name_str).expect("checked contains_id").collect();
                        if value_source == clap::parser::ValueSource::DefaultValue {
                            config_value
                                .map_or(matches_value, |m| m
                                    .into_iter()
                                    .collect::<Vec<(std::string::String, std::string::String)>>()
                                )
                        } else {
                            matches_value
                        }
                    } else {
                        config_value
                            .map(|h|
                                h.into_iter()
                                 .collect::<Vec<(std::string::String, std::string::String)>>()
                            ).unwrap_or_default()
                    }
                };
            }
        } else if strip_vec_wrapper_if_present(f).is_some() {
            // User-specified field's type was `Vec<T>`
            quote_spanned! {span=>
                let #name: #ty = {
                    let config_value: std::option::Option<#ty> = #config_value_expr;
                    if matches.contains_id(#name_str) {
                        let value_source = matches.value_source(#name_str).expect("checked contains_id");
                        let matches_value: #ty = matches.remove_many(#name_str).expect("checked contains_id").collect();
                        if value_source == clap::parser::ValueSource::DefaultValue {
                            config_value.unwrap_or(matches_value)
                        } else {
                            matches_value
                        }
                    } else {
                        config_value.unwrap_or_default()
                    }
                };
            }
        } else {
            quote_spanned! {span=>
                let #name: #ty = {
                    let config_value: std::option::Option<#ty> = #config_value_expr;
                    if matches.contains_id(#name_str) {
                        let value_source = matches.value_source(#name_str).expect("checked contains_id");
                        let matches_value: #ty = matches.remove_one(#name_str).expect("checked contains_id");
                        if value_source == clap::parser::ValueSource::DefaultValue {
                            config_value.unwrap_or(matches_value)
                        } else {
                            matches_value
                        }
                    } else {
                        config_value.expect(&format!("Required arg '{}' not provided in args or config.", #name_str))
                    }
                };
            }
        }
    });

    quote! {
        pub fn from_merged(
            mut matches: clap::ArgMatches,
            mut config: ::std::option::Option<#config_ident>
        ) -> Self {

            #(#field_updates)*

            Self {
                #(#struct_fields),*
            }
        }
    }
}

/**
Generate subcommand merging method that merges our config into the clap-generated enum, with precedence being:

- Things specified via `--arg` or `$ENV_VAR`
- Things in the config
- Clap defaults
*/
fn enum_merge_method(config_ident: &Ident, variants: &Punctuated<Variant, Comma>) -> TokenStream {
    let match_arms = variants.iter().map(|v| {
        let name = &v.ident;
        // TODO(gib): handle non-standard formats.
        let kebab_case_name = &name.to_string().as_str().to_kebab_case();
        let snake_case_ident = Ident::new(&name.to_string().as_str().to_snake_case(), name.span());
        let Some(f) = get_variant_field(v) else {
            // Unit variant has no fields, so just return it.
            return quote!(#kebab_case_name => Self::#name,);
        };
        let ty = &f.ty;

        let subcmd_opts_name = &ty;

        quote! {
            #kebab_case_name => Self::#name(
                #subcmd_opts_name::from_merged(matches,
                    config.and_then(|c| c.#snake_case_ident))
            ),
        }
    });

    quote! {
        pub fn from_merged(
            subcommand_name: String,
            mut matches: clap::ArgMatches,
            mut config: ::std::option::Option<#config_ident>
        ) -> Self {
            match subcommand_name.as_str() {
                #(#match_arms)*
                _ => unimplemented!("Should have exhaustively checked all possible subcommands."),
            }
        }
    }
}

// TODO(gib): steal from
// <https://stackoverflow.com/questions/55271857/how-can-i-get-the-t-from-an-optiont-when-using-syn>
// ?
/// If the field type is `Option<Foo>`, return `Some(Foo)`. Else return `None`.
fn strip_optional_wrapper_if_present(f: &Field) -> Option<&Type> {
    let ty = &f.ty;
    if let Type::Path(TypePath { path, .. }) = ty {
        if let Some(PathSegment { ident, arguments }) = path.segments.last() {
            if ident == &Ident::new("Option", f.span()) {
                if let PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                    args, ..
                }) = arguments
                {
                    if let Some(GenericArgument::Type(inner_type)) = args.first() {
                        return Some(inner_type);
                    }
                }
            }
        }
    }
    None
}

/// If the field type is `Vec<(String, String)>`, return `true`. Else return `false`.
fn is_vec_tuple_string(f: &Field) -> bool {
    let ty = &f.ty;
    fn path_is_ident(elem: Option<&Type>, ident: &Ident) -> bool {
        let Some(elem) = elem else { return false };
        if let Type::Path(TypePath { path, .. }) = elem {
            return path.is_ident(ident);
        }
        false
    }

    if let Type::Path(TypePath { path, .. }) = ty {
        if let Some(PathSegment { ident, arguments }) = path.segments.last() {
            if ident == &Ident::new("Vec", f.span()) {
                if let PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                    args, ..
                }) = arguments
                {
                    if let Some(GenericArgument::Type(Type::Tuple(TypeTuple { elems, .. }))) =
                        args.first()
                    {
                        let string_ident = Ident::new("String", f.span());
                        if path_is_ident(elems.first(), &string_ident)
                            && path_is_ident(elems.last(), &string_ident)
                        {
                            return true;
                        }
                    }
                }
            }
        }
    }
    false
}

/// If the field type is `Vec<Foo>`, return `Some(Foo)`. Else return `None`.
fn strip_vec_wrapper_if_present(f: &Field) -> Option<&Type> {
    let ty = &f.ty;

    if let Type::Path(TypePath { path, .. }) = ty {
        if let Some(PathSegment { ident, arguments }) = path.segments.last() {
            if ident == &Ident::new("Vec", f.span()) {
                if let PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                    args, ..
                }) = arguments
                {
                    if let Some(GenericArgument::Type(inner_type)) = args.first() {
                        return Some(inner_type);
                    }
                }
            }
        }
    }

    None
}

// Returns whether the field has a field attribute `#[clap(subcommand)]`.
fn is_subcommand_field(f: &Field) -> Result<bool, syn::Error> {
    let mut is_subcommand = false;
    'outer: for attr in f.attrs.iter() {
        if attr.path().is_ident("clap") {
            for meta in attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)? {
                if let Meta::Path(path) = meta {
                    if path.is_ident(&Ident::new("subcommand", path.span())) {
                        is_subcommand = true;
                        break 'outer;
                    }
                }
            }
        }
    }

    Ok(is_subcommand)
}

/// Check whether the user has asked us to skip generating/checking the config for this field.
fn is_field_marked_skipped(f: &Field) -> Result<bool, TokenStream> {
    for attr in f.attrs.iter() {
        if attr.path().is_ident(CLAP_CONFIG_ATTR_NAME) {
            let expr = attr
                .parse_args::<Expr>()
                .map_err(|e| e.into_compile_error())?;
            if let Expr::Path(ExprPath { path, .. }) = expr {
                if path.is_ident("skip") {
                    return Ok(true);
                } else {
                    return Err(syn::Error::new_spanned(
                        &attr.meta,
                        format!("expected `clap_config(skip)`, found {}", quote!(#attr)),
                    )
                    .into_compile_error());
                }
            }
        }
    }

    Ok(false)
}
