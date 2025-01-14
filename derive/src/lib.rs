extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields};

#[proc_macro_derive(StructMetadata)]
pub fn derive_struct_metadata(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let fields = if let Data::Struct(data_struct) = &input.data {
        if let Fields::Named(fields_named) = &data_struct.fields {
            &fields_named.named
        } else {
            panic!("StructMetadata can only be derived for structs with named fields");
        }
    } else {
        panic!("StructMetadata can only be derived for structs");
    };

    let field_collector = fields.iter().map(|field| {
        let field_name = &field.ident;
        let field_type = &field.ty;
        quote! {
            fields.insert(stringify!(#field_name).to_string(), stringify!(#field_type).to_string());
        }
    });

    let expanded = quote! {
        impl StructMetadata for #name {
            fn field_names_and_types() -> std::collections::BTreeMap<String, String> {
                let mut fields = std::collections::BTreeMap::new();
                #(#field_collector)*
                fields
            }
        }
    };

    TokenStream::from(expanded)
}