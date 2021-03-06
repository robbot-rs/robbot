use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Ident, Type};

pub(crate) fn expand_macro(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let mut field_types = Vec::new();
    let mut field_idents = Vec::new();

    match input.data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => {
                for f in fields.named.iter() {
                    field_types.push(f.ty.clone());
                    field_idents.push(f.ident.as_ref().unwrap().clone());
                }
            }
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    }

    let ident = input.ident;

    let storedata = expand_storedata(&ident, &field_idents, &field_types);
    let dataquery = expand_dataquery(&ident, &field_idents, &field_types);
    let dataquery_self = expand_dataquery_self(&ident, &field_idents, &field_types);
    let datadescriptor = expand_datadescriptor_self(&ident, &field_idents, &field_types);

    let expanded = quote! {
        #storedata
        #dataquery
        #dataquery_self
        #datadescriptor
    };

    proc_macro::TokenStream::from(expanded)
}

fn expand_storedata(ident: &Ident, field_idents: &[Ident], field_types: &[Type]) -> TokenStream {
    let trait_bounds = expand_type_trait_bounds(field_types);

    let resource_name = ident.to_string();

    let impl_serialize = field_idents.iter().map(|ident| {
        let name = ident.to_string();

        quote! {
            serializer.serialize_field(#name, &self.#ident)?;
        }
    });

    let impl_deserialize = field_idents.iter().map(|ident| {
        let name = ident.to_string();

        quote! {
            let #ident = deserializer.deserialize_field(#name)?;
        }
    });

    let dataquery_ident = Ident::new(&format!("{}Query", ident), Span::call_site());
    let datadescriptor_ident = Ident::new(&format!("{}Descriptor", ident), Span::call_site());

    quote! {
        impl<T> robbot::store::StoreData<T> for #ident
        where
            T: robbot::store::Store,
            #trait_bounds
        {
            type DataDescriptor = #datadescriptor_ident;
            type DataQuery = #dataquery_ident;

            fn resource_name() -> String {
                String::from(#resource_name)
            }

            fn serialize<S>(&self, serializer: &mut S) -> ::std::result::Result<(), S::Error>
            where
                S: robbot::store::Serializer<T>,
            {
                #(#impl_serialize)*
                ::std::result::Result::Ok(())
            }

            fn deserialize<D>(deserializer: &mut D) -> ::std::result::Result<Self, D::Error>
            where
                D: robbot::store::Deserializer<T>,
            {
                #(#impl_deserialize)*

                Ok(Self {
                    #(#field_idents,)*
                })
            }

            fn query() -> Self::DataQuery {
                #dataquery_ident::default()
            }
        }
    }
}

fn expand_dataquery(ident: &Ident, field_idents: &[Ident], field_types: &[Type]) -> TokenStream {
    let trait_bounds = expand_type_trait_bounds(field_types);

    let dataquery_ident = Ident::new(&format!("{}Query", ident), Span::call_site());

    let dataquery_fields = field_idents
        .iter()
        .zip(field_types.iter())
        .map(|(ident, ty)| {
            quote! {
                #ident: Option<#ty>,
            }
        });

    let dataquery_fns = field_idents
        .iter()
        .zip(field_types.iter())
        .map(|(ident, ty)| {
            quote! {
                pub fn #ident(mut self, t: #ty) -> Self {
                    self.#ident = ::std::option::Option::Some(t);
                    self
                }
            }
        });

    let impl_serialize = field_idents.iter().map(|ident| {
        let name = ident.to_string();

        quote! {
            {
                if let Some(val) = self.#ident.as_ref() {
                    serializer.serialize_field(#name, val)?;
                }
            }
        }
    });

    quote! {
        #[derive(Clone, Default)]
        pub struct #dataquery_ident {
            #(#dataquery_fields)*
        }

        impl #dataquery_ident {
            #(#dataquery_fns)*
        }

        impl<T> robbot::store::DataQuery<#ident, T> for #dataquery_ident
        where
            T: robbot::store::Store,
            #trait_bounds
        {
            fn serialize<S>(&self, serializer: &mut S) -> ::std::result::Result<(), S::Error>
            where
                S: robbot::store::Serializer<T>,
            {
                #(#impl_serialize)*

                ::std::result::Result::Ok(())
            }
        }
    }
}

fn expand_dataquery_self(
    ident: &Ident,
    field_idents: &[Ident],
    field_types: &[Type],
) -> TokenStream {
    let trait_bounds = expand_type_trait_bounds(field_types);

    let impl_serialize = field_idents.iter().map(|ident| {
        let name = ident.to_string();

        quote! {
            serializer.serialize_field(#name, &self.#ident)?;
        }
    });

    quote! {
        impl<T> robbot::store::DataQuery<#ident, T> for #ident
        where
            T: robbot::store::Store,
            #trait_bounds
        {
            fn serialize<S>(&self, serializer: &mut S) -> ::std::result::Result<(), S::Error>
            where
                S: robbot::store::Serializer<T>,
            {
                #(#impl_serialize)*

                ::std::result::Result::Ok(())
            }
        }
    }
}

/// Expand the required trait bounds for all unique types.
/// This includes the `Serialize<T>` and `Deserialize<T>` trait.
fn expand_type_trait_bounds(types: &[Type]) -> TokenStream {
    // Collect all unique types.
    let mut traits = Vec::new();
    for ty in types {
        if !traits.contains(ty) {
            traits.push(ty.clone());
        }
    }

    quote! {
        #(
            #traits: robbot::store::Serialize<T> + robbot::store::Deserialize<T>,
        )*
    }
}

fn expand_datadescriptor_self(
    ident: &Ident,
    field_idents: &[Ident],
    field_types: &[Type],
) -> TokenStream {
    let trait_bounds = expand_type_trait_bounds(field_types);

    let datadescriptor_ident = Ident::new(&format!("{}Descriptor", ident), Span::call_site());

    let impl_serialize = field_idents.iter().zip(field_types).map(|(ident, ty)| {
        let name = ident.to_string();
        let ty = ty.clone();

        quote! {
            serializer.serialize_field::<#ty>(#name)?;
        }
    });

    quote! {
        #[derive(Copy, Clone, Debug, Default)]
        pub struct #datadescriptor_ident;

        impl<T> robbot::store::DataDescriptor<#ident, T> for #datadescriptor_ident
        where
            T: robbot::store::Store,
            #trait_bounds
        {
            fn serialize<S>(&self, serializer: &mut S) -> ::std::result::Result<(), S::Error>
            where
                S: robbot::store::TypeSerializer<T>,
            {
                #(#impl_serialize)*

                ::std::result::Result::Ok(())
            }
        }
    }
}
