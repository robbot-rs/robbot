use std::sync::atomic::{AtomicUsize, Ordering};

use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::{quote, ToTokens};

static ID: AtomicUsize = AtomicUsize::new(0);

/// An internal (mangled) identifier.
#[derive(Clone, Debug)]
pub struct InternalIdent(Ident);

impl InternalIdent {
    pub fn new(ident: Ident) -> Self {
        let id = ID.fetch_add(1, Ordering::SeqCst);

        let ident = format!("__internal_robbot_{}_{}", id, ident);

        Self(Ident::new(&ident, Span::call_site()))
    }

    pub fn ident(&self) -> Ident {
        self.0.clone()
    }
}

impl ToTokens for InternalIdent {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let ident = &self.0;

        tokens.extend(quote! {
            #ident
        })
    }
}
