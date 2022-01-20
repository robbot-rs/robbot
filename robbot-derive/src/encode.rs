use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Ident, Index};

pub(crate) fn expand_macro(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let mut encode_calls = Vec::new();

    match input.data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => {
                for f in fields.named.iter() {
                    let ident = f.ident.clone().unwrap();

                    let expanded = quote! {
                        self.#ident.encode(encoder)?;
                    };

                    encode_calls.push(expanded);
                }
            }
            Fields::Unnamed(ref fields) => {
                for (i, _) in fields.unnamed.iter().enumerate() {
                    let index = Index::from(i);

                    let expanded = quote! {
                        self.#index.encode(encoder)?;
                    };

                    encode_calls.push(expanded);
                }
            }
            Fields::Unit => {}
        },
        Data::Enum(ref data) => {
            let mut match_arms = Vec::new();

            for (i, variant) in data.variants.iter().enumerate() {
                if i > u8::MAX as usize {
                    panic!("The maxium number of enum variants is {}", u8::MAX);
                }
                let i = i as u8;

                let variant_ident = variant.ident.clone();

                match variant.fields {
                    Fields::Named(ref fields) => {
                        let idents: Vec<Ident> = fields
                            .named
                            .iter()
                            .map(|f| f.ident.clone().unwrap())
                            .collect();

                        let expanded = quote! {
                            Self::#variant_ident{ #(#idents,)* } => {
                                #i.encode(encoder)?;
                                #(
                                    #idents.encode(encoder)?;
                                )*
                            }
                        };

                        match_arms.push(expanded);
                    }
                    Fields::Unnamed(ref fields) => {
                        let idents: Vec<Ident> = fields
                            .unnamed
                            .iter()
                            .enumerate()
                            .map(|(i, _)| Ident::new(&format!("_{}", i), Span::call_site()))
                            .collect();

                        let expanded = quote! {
                            Self::#variant_ident(#(#idents,)*) => {
                                #i.encode(encoder)?;
                                #(
                                    #idents.encode(encoder)?;
                                )*
                            }
                        };

                        match_arms.push(expanded);
                    }
                    Fields::Unit => {
                        let expanded = quote! {
                            Self::#variant_ident => {
                                #i.encode(encoder)?;
                            }
                        };

                        match_arms.push(expanded);
                    }
                }
            }

            let expanded = quote! {
                match self {
                    #(#match_arms)*
                }
            };

            encode_calls.push(expanded);
        }
        Data::Union(_) => unimplemented!(),
    }

    let ident = input.ident;

    let expanded = quote! {
        impl robbot::remote::Encode for #ident {
            fn encode<W>(&self, encoder: &mut robbot::remote::Encoder<W>) -> robbot::remote::Result<()>
                where W: ::std::io::Write
            {
                #(#encode_calls)*

                ::std::result::Result::Ok(())
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}
