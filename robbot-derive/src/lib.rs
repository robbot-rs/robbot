use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use std::collections::HashMap;
use syn::{
    parse::{Parse, ParseStream, Result},
    parse_macro_input,
    punctuated::Punctuated,
    Data, DeriveInput, Expr, Fields, Ident, ItemFn, Token,
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
        Some(expr) => quote! { cmd.#ident(#expr); },
        _ => unimplemented!(),
    });

    let expanded = quote! {
        #exec_fn

        pub fn #ident() -> crate::core::command::Command {
            let exec = crate::core::executor::Executor::from_fn(#command_ident);

            let cmd_exec = crate::core::command::CommandExecutor::Message(exec);

            let mut cmd = crate::core::command::Command::new(#ident_str);
            cmd.executor = Some(cmd_exec);

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

#[proc_macro_derive(StoreData)]
pub fn storedata(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // A list of types to be bound be the store.
    let mut types = Vec::new();
    let mut fields_rec = Vec::new();
    let mut fields_types = Vec::new();
    let mut fields_idents = Vec::new();

    let mut impl_deserialize = Vec::new();

    match input.data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => {
                for (i, f) in fields.named.iter().enumerate() {
                    if !types.contains(&f.ty) {
                        types.push(f.ty.clone());
                    }

                    let ident = &f.ident;
                    let name = ident.as_ref().unwrap().to_string();

                    let tokens = match i {
                        i if i == fields.named.len() - 1 => quote! {
                            serializer.serialize_field(#name, &self.#ident)
                        },
                        _ => quote! {
                            serializer.serialize_field(#name, &self.#ident)?;
                        },
                    };

                    impl_deserialize.push(match i {
                        i if i == fields.named.len() - 1 => quote! {
                            let #ident = deserializer.deserialize_field(#name)?;

                            Ok(Self {
                                #ident,
                                #(#fields_idents,)*
                            })
                        },
                        _ => quote! {
                            let #ident = deserializer.deserialize_field(#name)?;
                        },
                    });

                    fields_types.push(&f.ty);
                    fields_idents.push(&f.ident);

                    fields_rec.push(tokens);
                }
            }
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    }

    let recurse = types.iter().map(|ty| {
        quote! {
            #ty: crate::core::store::Serialize<T> + crate::core::store::Deserialize<T>,
        }
    });

    let dataquery_fields_recurse = fields_idents.iter().enumerate().map(|(i, ident)| {
        let ty = fields_types[i];

        quote! {
            #ident: ::std::option::Option<#ty>,
        }
    });

    let dataquery_fns_recurse = fields_idents.iter().enumerate().map(|(i, ident)| {
        let ty = fields_types[i];

        quote! {
            pub fn #ident(mut self, t: #ty) -> Self {
                self.#ident = ::std::option::Option::Some(t);
                self
            }
        }
    });

    let dataquery_serialize = fields_idents.iter().map(|ident| {
        let name = ident.as_ref().unwrap().to_string();

        quote! {
            {
                if let Some(val) = self.#ident.as_ref() {
                    serializer.serialize_field(#name, val)?;
                }
            }
        }
    });

    let ident = input.ident.clone();

    let dataquery_ident = Ident::new(&(input.ident.to_string() + "Query"), Span::call_site());

    let resource_name = input.ident.to_string().to_lowercase();

    let recurse2 = recurse.clone();

    let expanded = quote! {
        impl<T> crate::core::store::StoreData<T> for #ident
        where
            T: crate::core::store::Store,
            #(#recurse)*
        {
            type DataQuery = #dataquery_ident;

            fn resource_name() -> String {
                String::from(#resource_name)
            }

            fn serialize<S>(&self, serializer: &mut S) -> ::std::result::Result<(), S::Err>
            where
                S: crate::core::store::Serializer<T>,
            {
                #(#fields_rec)*
            }

            fn deserialize<D>(deserializer: &mut D) -> ::std::result::Result<Self, D::Err>
            where
                D: crate::core::store::Deserializer<T>,
            {
                #(#impl_deserialize)*
            }

            fn query() -> Self::DataQuery {
                #dataquery_ident::default()
            }
        }

        #[derive(Clone, Default)]
        pub struct #dataquery_ident {
            #(#dataquery_fields_recurse)*
        }

        impl #dataquery_ident {
            #(#dataquery_fns_recurse)*
        }

        impl<T> crate::core::store::DataQuery<#ident, T> for #dataquery_ident
        where
            T: crate::core::store::Store,
            #(#recurse2)*
        {
            fn serialize<S>(&self, serializer: &mut S) -> ::std::result::Result<(), S::Err>
            where
                S: crate::core::store::Serializer<T>,
            {
                #(#dataquery_serialize)*
                ::std::result::Result::Ok(())
            }
        }
    };

    TokenStream::from(expanded)
}
