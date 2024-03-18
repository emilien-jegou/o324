use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Meta};

#[proc_macro_derive(Document, attributes(document))]
pub fn document_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let fields = match input.data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields) => fields.named,
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    };

    let mut serialize_fields = Vec::new();
    let mut deserialize_fields_v = Vec::new();
    let mut deserialize_fields_asso = Vec::new();
    let mut deserialize_fields_build = Vec::new();
    let mut fields_names = Vec::new();
    let mut id_field_name = None;
    let mut fields_count = 0; // Initialize a counter for fields to be serialized

    for field in fields.iter() {
        let ident = &field.ident;
        let ty = &field.ty;

        let id_attr = field
            .attrs
            .iter()
            .find(|a| a.path.is_ident("document") && a.tokens.to_string().contains("id"));

        match id_attr {
            None => {
                serialize_fields.push(quote! {
                    SerializeStruct::serialize_field(&mut state, stringify!(#ident), &self.#ident)?;
                });
                deserialize_fields_v.push(quote! {
                    let mut #ident = None;
                });
                deserialize_fields_asso.push(quote! {
                    Field::#ident => {
                        if #ident.is_some() {
                            return Err(::serde::de::Error::duplicate_field(stringify!(#ident)));
                        }
                        let value = map.next_value::<#ty>()?;
                        #ident = Some(value);
                    },
                });
                deserialize_fields_build.push(quote! {
                    #ident: #ident.ok_or_else(|| ::serde::de::Error::missing_field(stringify!(#ident)))?,
                });
                fields_names.push(ident);
                fields_count += 1; // Increment the counter for each field to be serialized
            }
            Some(attr) => {
                let meta = attr.parse_meta().unwrap();
                if let Meta::List(meta_list) = meta {
                    for nested_meta in meta_list.nested {
                        if let syn::NestedMeta::Meta(Meta::Path(word)) = nested_meta {
                            if word.is_ident("id") {
                                id_field_name.clone_from(&field.ident);
                                break;
                            }
                        }
                    }
                };
            }
        }
    }

    let gen = quote! {
            impl ::serde::Serialize for #name {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                    where
                    S: ::serde::Serializer,
                    {
                        use ::serde::ser::SerializeStruct;
                        let mut state = serializer.serialize_struct(stringify!(#name), #fields_count as usize)?;
    #(#serialize_fields)*
                        SerializeStruct::end(state)
                    }
            }


            impl<'de> ::serde::Deserialize<'de> for #name {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: ::serde::Deserializer<'de>,
                {
                    #[allow(non_camel_case_types)]
                    #[derive(::serde_derive::Deserialize, Debug)]
                    #[serde(field_identifier, rename_all = "snake_case")]
                    #[doc(hidden)]
                    enum Field { #(#fields_names),* }

                    #[doc(hidden)]
                    struct Visitor;

                    impl<'de> ::serde::de::Visitor<'de> for Visitor {
                        type Value = #name;

                        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                            formatter.write_str(concat!("struct ", stringify!(#name)))
                        }

                        fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
                        where
                            V: ::serde::de::MapAccess<'de>,
                        {
                             #(#deserialize_fields_v)*
                             while let Some(key) = map.next_key::<Field>()? {
                                match key {
                                    #(#deserialize_fields_asso)*
                                    _ => return Err(::serde::de::Error::unknown_field(&format!("{:?}", key), FIELDS)),
                                }
                            }
                             Ok(#name {
                                #id_field_name: String::new(),
                                #(#deserialize_fields_build)*
                            })
                        }
                    }

                    const FIELDS: &'static [&'static str] = &[stringify!(#(#fields_names),*)];
                    deserializer.deserialize_struct(stringify!(#name), FIELDS, Visitor)
                }
            }

            impl Document for #name {
                fn get_document_id(&self) -> String { self.#id_field_name.clone() }
                fn set_document_id(&mut self, v: &str) { self.#id_field_name = v.to_string(); }
            }
        };

    TokenStream::from(gen)
}
