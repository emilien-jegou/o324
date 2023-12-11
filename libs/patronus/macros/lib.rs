extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, spanned::Spanned, Data, DeriveInput, Fields};

/// Usage:
/// ```rust
/// #[patronus("TaskUpdate")]
/// struct Task { id: String }
/// ```
#[proc_macro_attribute]
pub fn patronus(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    // "TaskUpdate"
    let name = parse_macro_input!(attr as syn::LitStr);

    // "Task"
    let struct_name = input.clone().ident;

    // `updated_struct_name` will be the identifier "TaskUpdate"
    let updated_struct_name = syn::Ident::new(&name.value(), struct_name.span());

    // Extract the fields of the "Task" struct
    let fields = if let Data::Struct(data) = input.clone().data {
        if let Fields::Named(fields) = data.fields {
            fields.named
        } else {
            panic!("the macro can only be applied on struct with named fields")
        }
    } else {
        panic!("the macro can only be applied on struct with named fields")
    };

    // Generate field definitions for TaskUpdate
    let field_definitions = fields.iter().map(|f| {
        let name = &f.ident; // "id"
        let ty = &f.ty; // String
        quote! {
            pub #name: ::patronus::Setter<#ty>,
        }
    });

    // Generate setter and reset methods for each field in TaskUpdate
    let setters = fields.iter().map(|f| {
        let name = &f.ident; // "id"
        let ty = &f.ty; // String

        // "set_id"
        let set_fn_name = syn::Ident::new(&format!("set_{}", name.as_ref().unwrap()), name.span());
        // "unset_id"
        let unset_fn_name =
            syn::Ident::new(&format!("unset_{}", name.as_ref().unwrap()), name.span());
        quote! {
            pub fn #set_fn_name(mut self, value: impl Into<#ty>) -> Self {
                self.#name = ::patronus::Setter::Set(value.into());
                self
            }

            pub fn #unset_fn_name(mut self) -> Self {
                self.#name = ::patronus::Setter::Unset;
                self
            }
        }
    });

    // The expanded code that will be inserted into the user's crate
    // This creates a new struct, TaskUpdate, based on Task
    let expanded = quote! {
        #input

        #[derive(Default)]
        pub struct #updated_struct_name {
            #(#field_definitions)*
        }

        impl #updated_struct_name {
            #(#setters)*
        }
    };

    TokenStream::from(expanded)
}
