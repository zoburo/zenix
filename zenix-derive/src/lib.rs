//! Proc-macro crate for Zenix — provides the [`FromRequest`] derive macro
//! that powers `#[zenix(from = "...")]` field-level deserialisation.

use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Lit, Meta, parse_macro_input};

/// Derive [`zenix::extract::FromRequest`] for a struct, enabling
/// `ctx.bind::<T>()` with per-field source annotations.
///
/// # Field annotations
///
/// | Annotation                 | Source                  |
/// |----------------------------|-------------------------|
/// | `#[zenix(from = "body")]`  | JSON request body       |
/// | `#[zenix(from = "query")]` | URL query string        |
/// | `#[zenix(from = "form")]`  | URL-encoded form body   |
///
/// If a field has no `#[zenix(from = ...)]` annotation, it defaults to `"body"`.
#[proc_macro_derive(FromRequest, attributes(zenix))]
pub fn derive_from_request(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let fields = match &input.data {
        Data::Struct(ds) => match &ds.fields {
            Fields::Named(nf) => &nf.named,
            _ => panic!("#[derive(FromRequest)] requires named fields"),
        },
        _ => panic!("#[derive(FromRequest)] only supports structs"),
    };

    // Classify fields by source.
    let mut body_fields: Vec<&syn::Ident> = Vec::new();
    let mut query_fields: Vec<&syn::Ident> = Vec::new();

    for field in fields {
        let Some(ident) = field.ident.as_ref() else {
            continue;
        };
        let source = parse_zenix_from_attr(&field.attrs).unwrap_or_else(|| "body".to_string());

        match source.as_str() {
            "body" | "form" => body_fields.push(ident),
            "query" => query_fields.push(ident),
            other => {
                panic!("unsupported `#[zenix(from = \"{other}\")]` — use body, query, or form")
            }
        }
    }

    let has_body = !body_fields.is_empty();
    let has_query = !query_fields.is_empty();

    // Body-field extraction tokens.
    let body_extractions: Vec<_> = body_fields
        .iter()
        .map(|ident| {
            let key = ident.to_string();
            quote! {
                #ident: ::serde_json::from_value(
                    __body_val.get(#key).cloned().unwrap_or(::serde_json::Value::Null)
                ).map_err(|e| {
                    ::zenix::extract::bad_request(format!("field `{}`: {}", #key, e))
                })?
            }
        })
        .collect();

    // Query-field extraction tokens.
    let query_extractions: Vec<_> = query_fields
        .iter()
        .map(|ident| {
            let key = ident.to_string();
            quote! {
                #ident: {
                    let __raw = __query_map.get(#key).map(|s| s.as_str()).unwrap_or("");
                    let __val: ::serde_json::Value =
                        ::serde_json::Value::String(__raw.to_string());
                    ::serde_json::from_value(__val).map_err(|e| {
                        ::zenix::extract::bad_request(format!("query `{}`: {}", #key, e))
                    })?
                }
            }
        })
        .collect();

    let expanded = if has_body && has_query {
        quote! {
            impl #impl_generics ::zenix::extract::FromRequest for #name #ty_generics #where_clause {
                fn from_request(
                    __ctx: &::zenix::Context,
                ) -> ::std::result::Result<Self, ::zenix::Response> {
                    let __body_val: ::serde_json::Value =
                        ::zenix::extract::parse_body_json(__ctx)?;
                    let __query_map: ::std::collections::HashMap<String, String> =
                        ::zenix::extract::parse_query(__ctx);

                    Ok(Self {
                        #(#body_extractions,)*
                        #(#query_extractions,)*
                    })
                }
            }
        }
    } else if has_body {
        quote! {
            impl #impl_generics ::zenix::extract::FromRequest for #name #ty_generics #where_clause {
                fn from_request(
                    __ctx: &::zenix::Context,
                ) -> ::std::result::Result<Self, ::zenix::Response> {
                    let __body_val: ::serde_json::Value =
                        ::zenix::extract::parse_body_json(__ctx)?;

                    Ok(Self {
                        #(#body_extractions,)*
                    })
                }
            }
        }
    } else if has_query {
        quote! {
            impl #impl_generics ::zenix::extract::FromRequest for #name #ty_generics #where_clause {
                fn from_request(
                    __ctx: &::zenix::Context,
                ) -> ::std::result::Result<Self, ::zenix::Response> {
                    let __query_map: ::std::collections::HashMap<String, String> =
                        ::zenix::extract::parse_query(__ctx);

                    Ok(Self {
                        #(#query_extractions,)*
                    })
                }
            }
        }
    } else {
        panic!("#[derive(FromRequest)] requires at least one field with a source annotation");
    };

    TokenStream::from(expanded)
}

/// Extracts the value of `#[zenix(from = "...")]` on a field, if present.
fn parse_zenix_from_attr(attrs: &[syn::Attribute]) -> Option<String> {
    for attr in attrs {
        if !attr.path().is_ident("zenix") {
            continue;
        }
        let meta = &attr.meta;
        if let Meta::List(list) = meta {
            let mut result = None;
            list.parse_nested_meta(|nested| {
                if nested.path.is_ident("from") {
                    let value: Lit = nested.value()?.parse()?;
                    if let Lit::Str(s) = value {
                        result = Some(s.value());
                    }
                }
                Ok(())
            })
            .ok();
            return result;
        }
    }
    None
}
