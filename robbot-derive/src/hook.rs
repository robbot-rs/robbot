use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, FnArg, Ident, ItemFn};

pub fn expand_macro(
    _attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as ItemFn);

    let ident = input.clone().sig.ident;
    let ident_str = ident.to_string();

    if input.sig.inputs.len() != 1 {
        panic!("Unrecognised hook event kind");
    }

    let context = match &input.sig.inputs[0] {
        FnArg::Receiver(_) => panic!("Unrecognised hook event kind"),
        FnArg::Typed(pat_type) => (*pat_type.ty).clone(),
    };

    let callback_ident = Ident::new(&format!("__hookcb_{}", ident), Span::call_site());
    let mut callback_fn = input;
    callback_fn.sig.ident = callback_ident.clone();

    let expanded = quote! {
        #callback_fn

        pub async fn #ident(state: &robbot_core::state::State) -> ::robbot::Result {
            use ::robbot::executor::Executor;
            use ::robbot::hook::{HookEvent, HookEventWrapper};

            let executor = robbot_core::executor::Executor::from_fn(#callback_ident);

            let hook = robbot_core::hook::Hook {
                name: #ident_str.to_string(),
                on_event: <#context as HookEventWrapper>::HookEvent::kind(),
            };

            let rx = state.hooks().add_hook(hook).await;

            robbot_core::hook::HookExecutor::new(rx, executor).run();

            Ok(())
        }
    };

    proc_macro::TokenStream::from(expanded)
}
