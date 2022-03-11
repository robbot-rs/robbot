use proc_macro::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::parse::{Parse, ParseStream, Result};
use syn::{braced, parse_macro_input, Expr, Path, Token, Type};

pub fn create(input: TokenStream) -> TokenStream {
    let QueryBuilder {
        store,
        datatype,
        filter,
    } = parse_macro_input!(input as QueryBuilder);

    let expanded = match filter {
        Some(_) => panic!("Use of create! with filtered query is unsupported"),
        None => quote! {
            {
                use ::robbot::store::Store;

                let descriptor = #store.make_descriptor::<#datatype>();

                #store.create(descriptor)
            }
        },
    };

    TokenStream::from(expanded)
}

pub fn delete(input: TokenStream) -> TokenStream {
    let QueryBuilder {
        store,
        datatype,
        filter,
    } = parse_macro_input!(input as QueryBuilder);

    let expanded = match filter {
        Some(filter) => quote! {
            {
                use ::robbot::store::Store;

                let query = #store.make_query::<#datatype>()#(.#filter)*;

                #store.delete(query)
            }
        },
        None => panic!("Use of delete! without a filtered query is currently not supported"),
    };

    TokenStream::from(expanded)
}

pub fn get(input: TokenStream) -> TokenStream {
    let builder = parse_macro_input!(input as QueryBuilder);

    TokenStream::from(builder.into_token_stream())
}

pub fn get_one(input: TokenStream) -> TokenStream {
    let QueryBuilder {
        store,
        datatype,
        filter,
    } = parse_macro_input!(input as QueryBuilder);

    let expanded = match filter {
        Some(filter) => quote! {
            {
                use ::robbot::store::Store;

                let descriptor = #store.make_descriptor::<#datatype>();
                let query  = #store.make_query::<#datatype>()#(.#filter)*;

                #store.get_one(descriptor, query)
            }
        },
        None => panic!("Use of get_one! without a filtered query is currently not supported"),
    };

    TokenStream::from(expanded)
}

struct QueryBuilder {
    store: Expr,
    datatype: Type,
    filter: Option<Vec<QueryFilter>>,
}

impl Parse for QueryBuilder {
    fn parse(input: ParseStream) -> Result<Self> {
        let store = input.parse()?;
        input.parse::<Token![,]>()?;
        let datatype = input.parse()?;

        let mut filter = None;
        if input.parse::<Token![=>]>().is_ok() {
            let content;
            braced!(content in input);

            filter = Some(Vec::new());
            loop {
                let val = content.parse()?;
                filter.as_mut().unwrap().push(val);

                if content.peek(Token![,]) {
                    content.parse::<Token![,]>()?;
                }

                if content.is_empty() {
                    break;
                }
            }
        }

        Ok(Self {
            store,
            datatype,
            filter,
        })
    }
}

impl ToTokens for QueryBuilder {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let store = self.store.clone();
        let datatype = self.datatype.clone();
        let filter = self.filter.clone();

        let tok = match filter {
            Some(filter) => quote! {
                {
                    use robbot::store::Store;

                    let descriptor = #store.make_descriptor::<#datatype>();
                    let query = #store.make_query::<#datatype>()#(.#filter)*;

                    #store.get(descriptor, query)
                }
            },
            None => quote! {
                {
                    use robbot::store::Store;

                    let descriptor = #store.make_descriptor::<#datatype>();

                    #store.get_all(descriptor)
                }
            },
        };

        tokens.append_all(tok);
    }
}

#[derive(Clone, Debug)]
struct QueryFilter {
    field: Path,
    value: Expr,
}

impl Parse for QueryFilter {
    fn parse(input: ParseStream) -> Result<Self> {
        let field = input.parse().unwrap();

        // Currently only equality is supported.
        input.parse::<Token![==]>()?;

        let value = input.parse()?;

        Ok(Self { field, value })
    }
}

impl ToTokens for QueryFilter {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let field = self.field.clone();
        let value = self.value.clone();

        tokens.append_all(quote! {
            #field(#value)
        });
    }
}
