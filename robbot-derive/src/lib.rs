mod decode;
mod encode;
mod hook;
mod ident;
mod kvmap;
mod module;
mod store;
mod storedata;
mod task;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use std::collections::HashMap;
use syn::{
    parse::{Parse, ParseStream, Result},
    parse_macro_input,
    punctuated::Punctuated,
    Expr, Ident, ItemFn, Token,
};

#[proc_macro_attribute]
pub fn command(attr: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as Args);
    let input = parse_macro_input!(input as ItemFn);

    let ident = input.clone().sig.ident;

    let ident_str = ident.to_string();

    let mut exec_fn = input;
    exec_fn.sig.ident = Ident::new(&format!("__{}", ident), Span::call_site());

    let command_ident = exec_fn.sig.ident.clone();

    let recurse = args.args.iter().map(|(ident, expr)| match expr {
        Some(expr) => {
            let ident = Ident::new(&format!("set_{}", ident), Span::call_site());

            quote! { cmd.#ident(#expr); }
        }
        _ => unimplemented!(),
    });

    let expanded = quote! {
        #exec_fn

        pub fn #ident() -> robbot_core::command::Command {
            use ::robbot::executor::Executor;

            let exec = robbot_core::executor::Executor::from_fn(#command_ident);

            let mut cmd = robbot_core::command::Command::new(#ident_str);
            cmd.executor = Some(exec);

            #(#recurse)*

            cmd
        }
    };

    TokenStream::from(expanded)
}

#[derive(Clone, Debug)]
struct Args {
    args: HashMap<Ident, Option<Expr>>,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> Result<Self> {
        let args = Punctuated::<Expr, Token![,]>::parse_terminated(input)?;

        let mut map = HashMap::new();

        for arg in args {
            match arg {
                Expr::Assign(expr) => {
                    let ident = match *expr.left {
                        Expr::Path(expr) => expr.path.segments.first().unwrap().ident.clone(),
                        _ => panic!("Invalid expr: {:?}", expr.left),
                    };

                    map.insert(ident, Some(*expr.right));
                }
                Expr::Path(expr) => {
                    let ident = expr.path.segments.first().unwrap().ident.clone();

                    map.insert(ident, None);
                }
                _ => panic!("Invalid expr: {:?}", arg),
            }
        }

        Ok(Self { args: map })
    }
}

#[proc_macro_attribute]
pub fn task(attr: TokenStream, input: TokenStream) -> TokenStream {
    task::task(attr, input)
}

#[proc_macro_attribute]
pub fn hook(attr: TokenStream, input: TokenStream) -> TokenStream {
    hook::expand_macro(attr, input)
}

#[proc_macro_derive(StoreData)]
pub fn storedata(input: TokenStream) -> TokenStream {
    storedata::expand_macro(input)
}

#[proc_macro_derive(Encode)]
pub fn encode(input: TokenStream) -> TokenStream {
    encode::expand_macro(input)
}

#[proc_macro_derive(Decode)]
pub fn decode(input: TokenStream) -> TokenStream {
    decode::expand_macro(input)
}

#[proc_macro]
pub fn module(input: TokenStream) -> TokenStream {
    module::expand_macro(input)
}

#[proc_macro]
pub fn create(input: TokenStream) -> TokenStream {
    store::create(input)
}

#[proc_macro]
pub fn delete(input: TokenStream) -> TokenStream {
    store::delete(input)
}

#[proc_macro]
pub fn get(input: TokenStream) -> TokenStream {
    store::get(input)
}

#[proc_macro]
pub fn get_one(input: TokenStream) -> TokenStream {
    store::get_one(input)
}

#[proc_macro]
pub fn insert(input: TokenStream) -> TokenStream {
    store::insert(input)
}
