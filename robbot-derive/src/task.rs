use std::time::Duration;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use quote::ToTokens;
use syn::parse::{Parse, ParseStream};
use syn::Expr;
use syn::Lit;
use syn::LitStr;
use syn::{parse_macro_input, ItemFn, Result};

use crate::ident::InternalIdent;
use crate::kvmap::KeyValueMap;

pub fn task(attr: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as Task);
    let input = parse_macro_input!(input as ItemFn);

    let fn_ident = input.clone().sig.ident;

    // Keep the same visibility.
    let fn_vis = input.clone().vis;

    let mut exec_fn = input;
    let exec_fn_ident = InternalIdent::new(exec_fn.sig.ident);
    exec_fn.sig.ident = exec_fn_ident.ident();

    let name = args.name.unwrap_or(fn_ident.to_string());
    let schedule = args.schedule.unwrap();
    let on_load = args.on_load.unwrap_or(false);

    let expanded = quote! {
        #fn_vis fn #fn_ident() -> ::robbot_core::task::Task {
            #exec_fn

            use ::robbot::executor::Executor as _;

            ::robbot_core::task::Task {
                name: ::std::string::String::from(#name),
                schedule: #schedule,
                on_load: #on_load,
                executor: ::robbot_core::executor::Executor::from_fn(#exec_fn_ident),
            }
        }
    };

    TokenStream::from(expanded)
}

#[derive(Debug, Default)]
struct Task {
    name: Option<String>,
    schedule: Option<TaskSchedule>,
    on_load: Option<bool>,
}

impl Parse for Task {
    fn parse(input: ParseStream) -> Result<Self> {
        let args = KeyValueMap::parse(input)?;

        let name = args.get("name").map(|expr| match expr {
            Expr::Lit(lit) => match &lit.lit {
                Lit::Str(s) => s.value(),
                _ => panic!("Expected string literal"),
            },
            _ => panic!("Expected literal"),
        });

        // Schedule
        let mut schedule = None;
        if let Some(arg) = args.get("interval") {
            let interval = syn::parse2::<IntervalSchedule>(arg.into_token_stream())?;
            schedule = Some(TaskSchedule::Interval(interval.interval));
        }

        // TODO: IMPL
        if let Some(_) = args.get("at") {
            schedule = Some(TaskSchedule::At(()));
        }

        assert!(schedule.is_some());

        // on load
        let on_load = args.get("on_load").map(|expr| match expr {
            Expr::Lit(lit) => match &lit.lit {
                Lit::Bool(val) => val.value(),
                _ => panic!("Expected bool literal"),
            },
            _ => panic!("Expected literal"),
        });

        Ok(Self {
            name,
            schedule,
            on_load,
        })
    }
}

/// A single time span component (e.g.`1s`).
#[derive(Clone, Debug)]
struct DateTimeComponent {
    num: u64,
    unit: String,
}

impl DateTimeComponent {
    pub fn duration(self) -> Duration {
        let secs = match self.unit.as_str() {
            "s" => self.num,
            "m" => self.num * 60,
            "h" => self.num * 60 * 60,
            "d" => self.num * 60 * 60 * 24,
            "w" => self.num * 60 * 60 * 24 * 7,
            _ => panic!("Unknown time unit"),
        };

        Duration::from_secs(secs)
    }
}

impl Parse for DateTimeComponent {
    fn parse(input: ParseStream) -> Result<Self> {
        let lit = input.parse::<LitStr>()?.value();

        // Ending index of the integer literal (exlusive).
        let end = lit
            .chars()
            .enumerate()
            .find(|(_, c)| !c.is_ascii_digit())
            .map(|(i, _)| i);

        if end.is_none() {
            panic!("Missing time unit");
        }
        let end = end.unwrap();

        let num: u64 = lit[..end].parse().unwrap();
        let unit: String = lit[end..].to_string();

        Ok(Self { num, unit })
    }
}

#[derive(Debug)]
enum TaskSchedule {
    Interval(Duration),
    At(()),
}

impl ToTokens for TaskSchedule {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let tt = match self {
            Self::Interval(dur) => {
                let secs: i64 = dur.as_secs().try_into().unwrap();

                quote! {
                    ::robbot::task::TaskSchedule::Interval(::chrono::Duration::seconds(#secs))
                }
            }
            _ => unimplemented!(),
        };

        tokens.extend(tt);
    }
}

struct IntervalSchedule {
    interval: Duration,
}

impl Parse for IntervalSchedule {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut components = Vec::new();

        while !input.is_empty() {
            components.push(DateTimeComponent::parse(input)?);
        }

        let mut interval = Duration::new(0, 0);
        for comp in components {
            interval += comp.duration();
        }

        Ok(Self { interval })
    }
}
