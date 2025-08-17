extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, spanned::Spanned, Data, DeriveInput, Fields};

/// Usage:
/// `
/// #[patronus(name = "TaskUpdate", derives = "Clone")]
/// struct Task { id: String }
#[proc_macro_attribute]
pub fn patronus(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let attr_args = parse_macro_input!(attr as syn::AttributeArgs);

    let struct_name = input.clone().ident;
    let mut derives = Vec::new();
    let mut updated_struct_name = None;

    for arg in attr_args {
        match arg {
            syn::NestedMeta::Meta(syn::Meta::NameValue(nv)) if nv.path.is_ident("name") => {
                if let syn::Lit::Str(lit) = nv.lit {
                    updated_struct_name = Some(syn::Ident::new(&lit.value(), struct_name.span()));
                }
            },
            syn::NestedMeta::Meta(syn::Meta::NameValue(nv)) if nv.path.is_ident("derives") => {
                if let syn::Lit::Str(lit) = nv.lit {
                    derives = lit.value().split(',').map(|s| s.trim().to_string()).collect();
                }
            },
            _ => {}
        }
    }
    //
    // `updated_struct_name` will be the identifier "TaskUpdate"
    let updated_struct_name = updated_struct_name.expect("missing 'name' field");

    let derives_tokens = derives.iter().map(|d| {
        syn::Ident::new(d, struct_name.span())
    });

    let fields = if let Data::Struct(data) = input.clone().data {
        if let Fields::Named(fields) = data.fields {
            fields.named
        } else {
            panic!("the macro can only be applied on struct with named fields")
        }
    } else {
        panic!("the macro can only be applied on struct with named fields")
    };

    let field_definitions = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        quote! {
            pub #name: Option<#ty>,
        }
    });

    let setters = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        let set_opt_fn_name = syn::Ident::new(&format!("set_opt_{}", name.as_ref().unwrap()), name.span());
        let set_fn_name = syn::Ident::new(&format!("set_{}", name.as_ref().unwrap()), name.span());
        let unset_fn_name = syn::Ident::new(&format!("unset_{}", name.as_ref().unwrap()), name.span());
        quote! {
            #[allow(non_snake_case)]
            pub fn #set_opt_fn_name(mut self, value: impl Into<Option<#ty>>) -> Self {
                self.#name = value.into();
                self
            }

            #[allow(non_snake_case)]
            pub fn #set_fn_name(mut self, value: impl Into<#ty>) -> Self {
                self.#name = Some(value.into());
                self
            }

            #[allow(non_snake_case)]
            pub fn #unset_fn_name(mut self) -> Self {
                self.#name = None;
                self
            }
        }
    });

    let expanded = quote! {
        #input

        #[derive(#(#derives_tokens),*)]
        pub struct #updated_struct_name {
            #(#field_definitions)*
        }

        impl #updated_struct_name {
            #(#setters)*
        }
    };

    TokenStream::from(expanded)
}
