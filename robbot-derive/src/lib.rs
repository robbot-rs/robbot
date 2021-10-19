use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, Ident, ItemFn};

#[proc_macro_attribute]
pub fn command(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);

    let ident = input.clone().sig.ident;

    let ident_str = ident.to_string();

    let mut exec_fn = input.clone();
    exec_fn.sig.ident = Ident::new(&format!("__{}", ident), Span::call_site());

    let command_ident = exec_fn.sig.ident.clone();

    let expanded = quote! {
        #exec_fn

        pub fn #ident() -> crate::core::command::Command {
            let exec = crate::core::executor::Executor::from_fn(#command_ident);

            let cmd_exec = crate::core::command::CommandExecutor::Message(exec);

            let mut cmd = crate::core::command::Command::new(#ident_str);
            cmd.executor = Some(cmd_exec);

            cmd
        }
    };

    TokenStream::from(expanded)
}
