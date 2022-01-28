use proc_macro2::Literal;
use proc_macro2::TokenStream;
use quote::quote;
use quote::{ToTokens, TokenStreamExt};
use syn::parse::{Parse, ParseStream, Result};
use syn::{braced, bracketed, parse_macro_input, Expr, ExprPath, Ident, Token, Type};

struct Module {
    // This should a &str or a ToString type.
    _name: Expr,
    cmds: CommandMap,
    store: StoreDataTypes,
    tasks: Tasks,
}

impl Parse for Module {
    fn parse(input: ParseStream) -> Result<Self> {
        let pair = input.parse::<KeyValuePair<Ident, Expr>>()?;
        if pair.key != "name" {
            panic!("First key needs to be 'name'");
        }
        let _name = pair.value;

        let pair = input.parse::<KeyValuePair<Ident, CommandMap>>()?;
        if pair.key != "cmds" {
            panic!("Second key needs to be 'cmds'");
        }
        let cmds = pair.value;

        let mut store = StoreDataTypes::default();
        let pair = input.parse::<KeyValuePair<Ident, StoreDataTypes>>();
        if let Ok(pair) = pair {
            if pair.key != "store" {
                panic!("Third key needs to be 'store'");
            }
            store = pair.value;
        }

        let mut tasks = Tasks::default();
        let pair = input.parse::<KeyValuePair<Ident, Tasks>>();
        if let Ok(pair) = pair {
            if pair.key != "tasks" {
                panic!("Fourth key needs to be 'tasks'");
            }
            tasks = pair.value;
        }

        Ok(Self {
            _name,
            cmds,
            store,
            tasks,
        })
    }
}

#[derive(Clone, Debug)]
struct KeyValuePair<K, V>
where
    K: Parse,
    V: Parse,
{
    key: K,
    value: V,
}

impl<K, V> Parse for KeyValuePair<K, V>
where
    K: Parse,
    V: Parse,
{
    fn parse(input: ParseStream) -> Result<Self> {
        let key = input.parse()?;
        input.parse::<Token![:]>()?;
        let value = input.parse()?;
        input.parse::<Token![,]>()?;

        Ok(Self { key, value })
    }
}

pub fn expand_macro(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let module = parse_macro_input!(input as Module);

    let cmds = module.cmds;
    let store = module.store;
    let tasks = module.tasks;

    let expanded = quote! {
        pub async fn init(state: &::robbot_core::state::State) -> ::robbot::Result {
            for cmd in #cmds {
                state.commands().load_command(cmd, None)?;
                #store
                #tasks
            }

            Ok(())
        }
    };

    proc_macro::TokenStream::from(expanded)
}

enum CommandMap {
    List(Vec<ExprPath>),
    Command(Literal, Box<Self>),
}

impl Parse for CommandMap {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        braced!(content in input);

        let literal = content.step(|cursor| match cursor.literal() {
            Some((literal, cursor)) => Ok((literal, cursor)),
            None => Err(cursor.error("")),
        });

        match literal {
            // Construct a [`Self::Command`]
            Ok(literal) => {
                content.parse::<Token![:]>()?;
                let inner = Self::parse(&content)?;
                content.parse::<Token![,]>()?;

                Ok(Self::Command(literal, Box::new(inner)))
            }
            // Construct a [`Self::List`]
            Err(_) => {
                let mut cmds = Vec::new();

                while !content.is_empty() {
                    let path = content.parse::<ExprPath>().unwrap();
                    content.parse::<Token![,]>()?;
                    cmds.push(path);
                }

                Ok(Self::List(cmds))
            }
        }
    }
}

impl ToTokens for CommandMap {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let token = match self {
            Self::List(vec) => {
                quote! {
                    {
                        [#(#vec()),*]
                    }
                }
            }
            Self::Command(literal, map) => {
                quote! {
                    {
                        let mut command = ::robbot_core::command::Command::new(#literal);

                        for cmd in #map {
                            command.sub_commands.insert(cmd);
                        }

                        [command]
                    }
                }
            }
        };

        tokens.append_all(&[token]);
    }
}

#[derive(Clone, Debug, Default)]
struct StoreDataTypes {
    types: Vec<Type>,
}

impl Parse for StoreDataTypes {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        bracketed!(content in input);

        let mut types = Vec::new();
        while !content.is_empty() {
            let ty = content.parse::<Type>()?;
            content.parse::<Token![,]>()?;
            types.push(ty);
        }

        Ok(Self { types })
    }
}

impl ToTokens for StoreDataTypes {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let types = self.types.clone();

        let token = match types.len() {
            0 => quote! {{}},
            _ => quote! {
                let res = ::tokio::try_join! {
                    #(
                        state.store().create::<#types>(),
                    )*
                };
                res?;
            },
        };

        tokens.append_all(&[token]);
    }
}

#[derive(Clone, Debug, Default)]
struct Tasks {
    tasks: Vec<ExprPath>,
}

impl Parse for Tasks {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        bracketed!(content in input);

        let mut tasks = Vec::new();
        while !content.is_empty() {
            let path = content.parse()?;
            content.parse::<Token![,]>()?;
            tasks.push(path);
        }

        Ok(Self { tasks })
    }
}

impl ToTokens for Tasks {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let tasks = self.tasks.clone();

        let token = match tasks.len() {
            0 => quote! {{}},
            _ => quote! {
                let res = ::tokio::try_join! {
                    #(
                        state.tasks().add_task(#tasks),
                    )*
                };
                res?;
            },
        };

        tokens.append_all(&[token]);
    }
}