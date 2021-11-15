use proc_macro2::TokenStream;
use quote::*;
use syn::DeriveInput;

use crate::common::struct_fields;

pub fn impl_db_mirror(ast: &DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let idents = struct_fields(ast)
        .iter()
        .map(|f| f.ident.clone().unwrap())
        .collect::<Vec<_>>();
    let idents_copy = idents.clone();

    let fields = idents
        .iter()
        .map(|i| i.to_string())
        .collect::<Vec<String>>();
    let names = fields.join(", ");
    let question_marks = fields
        .iter()
        .map(|_| "?".to_string())
        .collect::<Vec<String>>()
        .join(", ");

    quote! {
        impl #name {
            pub fn insert_query() -> &'static str {
                concat!("insert into ", stringify!(#name), "(",
                  #names,
                 ") values (",
                 #question_marks,
                 ")")
            }

            pub fn into_query_values(self) -> cassandra_protocol::query::QueryValues {
                use std::collections::HashMap;
                let mut values: HashMap<String, cassandra_protocol::types::value::Value> = HashMap::new();

                #(
                    values.insert(stringify!(#idents).to_string(), self.#idents_copy.into());
                )*

                cassandra_protocol::query::QueryValues::NamedValues(values)
            }
        }
    }
}
