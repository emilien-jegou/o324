use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, punctuated::Punctuated, Ident, ItemStruct, Token, TypePath};

#[proc_macro_attribute]
pub fn wrap_builder(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_struct = parse_macro_input!(item as ItemStruct);
    let original_struct_name = &input_struct.ident;

    // Parse the attribute arguments for the wrapper type (e.g., Arc, Rc)
    let wrapper_type_path = if attr.is_empty() {
        // Default to Arc if no argument is provided
        TypePath {
            qself: None,
            path: Ident::new("Arc", proc_macro2::Span::call_site()).into(),
        }
    } else {
        // Parse the attribute as a single TypePath (e.g., Arc, Rc)
        let parser = Punctuated::<TypePath, Token![,]>::parse_terminated;
        let parsed_attr = parse_macro_input!(attr with parser);

        if parsed_attr.len() != 1 {
            return syn::Error::new_spanned(
                parsed_attr,
                "Expected a single type path (e.g., `Arc`, `Rc`) as an argument.",
            )
            .to_compile_error()
            .into();
        }
        parsed_attr.into_iter().next().unwrap()
    };

    let mut inner_struct = input_struct.clone(); // Clone to modify for the Inner struct
    let inner_struct_name = Ident::new(
        &format!("{original_struct_name}Inner"),
        original_struct_name.span(),
    );
    let builder_name = Ident::new(
        &format!("{original_struct_name}InnerBuilder"),
        original_struct_name.span(),
    );

    // Rename the cloned struct to the "Inner" struct
    inner_struct.ident = inner_struct_name.clone();

    // Extract generics from the original struct for proper propagation
    let (impl_generics, ty_generics, where_clause) = input_struct.generics.split_for_impl();

    let output = quote! {
        // The generated Inner struct with TypedBuilder
        #[derive(typed_builder::TypedBuilder)]
        #[builder(build_method(into = #original_struct_name #ty_generics))]
        #inner_struct // This now refers to the modified inner_struct (e.g., TaskRepositoryInner)

        // The public wrapper struct
        #[derive(Clone, derive_more::Deref)]
        #[deref(forward)]
        pub struct #original_struct_name #impl_generics (
            #wrapper_type_path #ty_generics <#inner_struct_name #ty_generics>
        ) #where_clause;

        impl #impl_generics #original_struct_name #ty_generics #where_clause {
            pub fn builder() -> #builder_name #ty_generics {
                #inner_struct_name #ty_generics ::builder()
            }
        }

        // Implementation of From for conversion
        impl #impl_generics From<#inner_struct_name #ty_generics> for #original_struct_name #ty_generics #where_clause {
            fn from(inner: #inner_struct_name #ty_generics) -> Self {
                #original_struct_name (#wrapper_type_path::new(inner))
            }
        }
    };

    output.into()
}
