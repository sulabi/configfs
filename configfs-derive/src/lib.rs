use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, LitStr, parse_macro_input};

#[proc_macro_derive(Config, attributes(config))]
pub fn derive_config(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    if !input.generics.params.is_empty() {
        return syn::Error::new_spanned(
            &input.generics,
            "#[derive(Config)] doesn't support generic structs",
        )
        .to_compile_error()
        .into();
    }

    if let Some(w) = input.generics.where_clause {
        return syn::Error::new_spanned(w, "#[derive(Config)] doesn't support where clauses")
            .to_compile_error()
            .into();
    }

    let mut system: Option<LitStr> = None;
    let mut path: Option<LitStr> = None;

    for attr in &input.attrs {
        if !attr.path().is_ident("config") {
            continue;
        }

        let res = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("system") {
                system = Some(meta.value()?.parse()?);
                Ok(())
            } else if meta.path.is_ident("path") {
                path = Some(meta.value()?.parse()?);
                Ok(())
            } else {
                Err(meta.error("Expected `system` or `path`"))
            }
        });

        if let Err(e) = res {
            return e.to_compile_error().into();
        }
    }

    let dir = match (system, path) {
        (Some(s), None) => quote!(::configfs::ConfigPath::System(#s)),
        (None, Some(p)) => {
            quote!(::configfs::ConfigPath::Custom(::std::path::PathBuf::from(#p)))
        }
        _ => {
            return syn::Error::new_spanned(
                name,
                "Use exactly one of #[config(system = \"...\")] or #[config(path = \"...\")]",
            )
            .to_compile_error()
            .into();
        }
    };

    quote! {
        impl ::configfs::ConfigFile for #name {
            fn config_directory() -> ::configfs::ConfigPath { #dir }

            fn global() -> &'static ::std::sync::OnceLock::<::std::sync::RwLock<Self>> {
                static CELL: ::std::sync::OnceLock<::std::sync::RwLock<#name>> = ::std::sync::OnceLock::new();
                &CELL
            }
        }
    }
    .into()
}
