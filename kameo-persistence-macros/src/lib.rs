use heck::ToShoutySnakeCase;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(PersistentActor, attributes(snapshot))]
pub fn derive_persistent_actor(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    let snapshot_type = find_snapshot_type(&input);

    let regiestry_ident = syn::Ident::new(
        &format!("{}_REGISTRY", name.to_string().to_shouty_snake_case()),
        name.span(),
    );

    let expanded = quote! {

        static #regiestry_ident: ::std::sync::LazyLock<::std::sync::RwLock<::kameo_persistence::BiHashMap<::url::Url, ::kameo::prelude::WeakActorRef<#name>>>> =
            ::std::sync::LazyLock::new(|| ::std::sync::RwLock::new(::kameo_persistence::BiHashMap::new()));


        impl ::kameo_persistence::PersistentActor for #name {
            type Snapshot = #snapshot_type;


            fn register_persistent(persistence_key: ::url::Url, actor_ref: &::kameo::prelude::ActorRef<Self>) -> ::anyhow::Result<()> {
                let Ok(mut registry) = #regiestry_ident.write() else {
                    ::anyhow::bail!("Failed to acquire write lock on registry");
                };
                if let Some(old_pair) = registry.insert(persistence_key, actor_ref.downgrade()) {
                    #[cfg(feature = "tracing")]
                    ::tracing::warn!("Existing persistent actor reference for {old_pair:?} is replaced");
                }
                Ok(())
            }

            fn persistence_key(actor_ref: &::kameo::prelude::ActorRef<Self>) -> Option<::url::Url> {
                let registry = #regiestry_ident.read().unwrap();
                registry.get_left(&actor_ref.downgrade()).cloned()
            }

            fn lookup_persistent(persistence_key: &::url::Url) -> Option<::kameo::prelude::ActorRef<Self>> {
                let registry = #regiestry_ident.read().unwrap();
                registry
                    .get_right(persistence_key)
                    .and_then(|weak_ref| weak_ref.upgrade())
            }
        }
    };

    TokenStream::from(expanded)
}

fn find_snapshot_type(input: &DeriveInput) -> syn::Type {
    // Look for #[snapshot(Type)] attribute
    for attr in &input.attrs {
        if attr.path().is_ident("snapshot") {
            if let Ok(snapshot_type) = attr.parse_args::<syn::Type>() {
                return snapshot_type;
            }
        }
    }

    syn::parse_quote! { <Self as ::kameo::prelude::Actor>::Args }
}
