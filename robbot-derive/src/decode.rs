use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Ident};

pub(crate) fn expand_macro(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let fn_body = match input.data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => {
                let idents: Vec<Ident> = fields
                    .named
                    .iter()
                    .map(|f| f.ident.clone().unwrap())
                    .collect();

                let recurse = fields.named.iter().map(|f| {
                    let ident = f.ident.clone().unwrap();
                    let ty = f.ty.clone();

                    quote! {
                        let #ident = <#ty as robbot::remote::Decode>::decode(decoder)?;
                    }
                });

                quote! {
                    #(#recurse)*

                    Ok(Self { #(#idents,)* })
                }
            }
            Fields::Unnamed(ref fields) => {
                let idents: Vec<Ident> = fields
                    .unnamed
                    .iter()
                    .enumerate()
                    .map(|(i, _)| Ident::new(&format!("_{}", i), Span::call_site()))
                    .collect();

                let recurse = fields.unnamed.iter().enumerate().map(|(i, f)| {
                    let ident = Ident::new(&format!("_{}", i), Span::call_site());
                    let ty = f.ty.clone();

                    quote! {
                        let #ident = <#ty as robbot::remote::Decode>::decode(decoder)?;
                    }
                });

                quote! {
                    #(#recurse)*

                    Ok(Self(#(#idents)*))
                }
            }
            Fields::Unit => {
                quote! {
                    Self;
                }
            }
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

                        let recurse = fields.named.iter().map(|f| {
                            let ident = f.ident.clone().unwrap();
                            let ty = f.ty.clone();

                            quote! {
                                let #ident = <#ty as robbot::remote::Decode>::decode(decoder)?;
                            }
                        });

                        let expanded = quote! {
                            #i => {
                                #(#recurse)*

                                Self::#variant_ident { #(#idents,)* }
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

                        let recurse = fields.unnamed.iter().enumerate().map(|(i, f)| {
                            let ident = Ident::new(&format!("_{}", i), Span::call_site());
                            let ty = f.ty.clone();

                            quote! {
                                let #ident = <#ty as robbot::remote::Decode>::decode(decoder)?;
                            }
                        });

                        let expanded = quote! {
                            #i => {
                                #(#recurse)*

                                Self::#variant_ident(#(#idents,)*)
                            }
                        };

                        match_arms.push(expanded);
                    }
                    Fields::Unit => {
                        let expanded = quote! {
                            #i => Self::#variant_ident,
                        };

                        match_arms.push(expanded);
                    }
                }
            }

            quote! {
                let enum_variant = <u8 as robbot::remote::Decode>::decode(decoder)?;

                Ok(match enum_variant {
                    #(#match_arms)*
                    _ => unreachable!(),
                })
            }
        }
        Data::Union(_data) => {
            unimplemented!();
        }
    };

    let ident = input.ident;

    let expanded = quote! {
        impl robbot::remote::Decode for #ident {
            fn decode<R>(decoder: &mut robbot::remote::Decoder<R>) -> robbot::remote::Result<Self>
                where R: ::std::io::Read
            {
                #fn_body
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}
