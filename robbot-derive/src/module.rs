use proc_macro2::Literal;
use proc_macro2::TokenStream;
use quote::quote;
use quote::{ToTokens, TokenStreamExt};
use syn::parse::{Parse, ParseStream, Result};
use syn::{braced, bracketed, parse_macro_input, Expr, ExprPath, Ident, Path, Token, Type};

pub fn expand_macro(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let module = parse_macro_input!(input as Module);

    proc_macro::TokenStream::from(module.into_token_stream())
}

#[derive(Debug)]
struct Module {
    // This should a &str or a ToString type.
    name: Expr,
    commands: Option<CommandMap>,
    store: Option<StoreDataTypes>,
    tasks: Option<Tasks>,
    hooks: Option<Hooks>,
}

impl Parse for Module {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut name: Option<Expr> = None;
        let mut commands: Option<CommandMap> = None;
        let mut store: Option<StoreDataTypes> = None;
        let mut tasks: Option<Tasks> = None;
        let mut hooks: Option<Hooks> = None;

        for _ in ["name", "cmds", "store", "tasks", "hooks"] {
            if input.is_empty() {
                break;
            }

            let ident: Ident = input.fork().parse()?;

            match ident.to_string().as_str() {
                "name" => name = Some(input.parse::<KeyValuePair<Ident, Expr>>()?.into_value()),
                "cmds" => {
                    commands = Some(
                        input
                            .parse::<KeyValuePair<Ident, CommandMap>>()?
                            .into_value(),
                    )
                }
                "store" => {
                    store = Some(
                        input
                            .parse::<KeyValuePair<Ident, StoreDataTypes>>()?
                            .into_value(),
                    )
                }
                "tasks" => tasks = Some(input.parse::<KeyValuePair<Ident, Tasks>>()?.into_value()),
                "hooks" => hooks = Some(input.parse::<KeyValuePair<Ident, Hooks>>()?.into_value()),
                _ => panic!("Invalid key: {}", ident),
            }
        }

        // A name always needs to be given.
        let name = match name {
            Some(name) => name,
            None => panic!("No name value given for module"),
        };

        Ok(Self {
            name,
            commands,
            store,
            tasks,
            hooks,
        })
    }
}

impl ToTokens for Module {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            name,
            commands,
            store,
            tasks,
            hooks,
        } = self;

        let output = quote! {
            pub async fn init(state: &robbot_core::state::State) -> robbot::Result {
                let module = robbot_core::module::Module {
                    name: #name.to_string(),
                    commands: std::collections::HashSet::new(),
                };

                let id = state.modules().add_module(module)?;

                for cmd in #commands {
                    let options = robbot_core::command::AddOptions::new().module_id(id);

                    state.commands().add_commands([cmd], options)?;
                }

                #store
                #tasks
                #hooks

                Ok(())
            }
        };

        tokens.append_all(output)
    }
}

/// A single key-value pair in the format `name: key`.
#[derive(Clone, Debug)]
struct KeyValuePair<K, V>
where
    K: Parse,
    V: Parse,
{
    _key: K,
    value: V,
}

impl<K, V> KeyValuePair<K, V>
where
    K: Parse,
    V: Parse,
{
    fn into_value(self) -> V {
        self.value
    }
}

impl<K, V> Parse for KeyValuePair<K, V>
where
    K: Parse,
    V: Parse,
{
    fn parse(input: ParseStream) -> Result<Self> {
        let _key = input.parse()?;
        input.parse::<Token![:]>()?;
        let value = input.parse()?;

        // Parse optional `,` token at the end.
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        }

        Ok(Self { _key, value })
    }
}

#[derive(Debug)]
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

#[derive(Clone, Debug, Default)]
struct Hooks {
    hooks: Vec<Path>,
}

impl Parse for Hooks {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        bracketed!(content in input);

        let mut hooks = Vec::new();
        while !content.is_empty() {
            let path = content.parse()?;
            content.parse::<Token![,]>()?;
            hooks.push(path);
        }

        Ok(Self { hooks })
    }
}

impl ToTokens for Hooks {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let hooks = self.hooks.clone();

        let token = match hooks.len() {
            0 => quote! {{}},
            _ => quote! {
                let res = ::tokio::try_join! {
                    #(
                        #hooks(&state),
                    )*
                };
                res?;
            },
        };

        tokens.append_all(&[token]);
    }
}
