use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Fields, Ident, ItemEnum, Type};

// A helper struct to keep variant information organized.
struct PackedInfo<'a> {
    name: &'a Ident,
    ty: &'a Type,
    field_name: Ident,
}

#[proc_macro_attribute]
pub fn dyn_variant(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_enum = parse_macro_input!(item as ItemEnum);

    let enum_name = &input_enum.ident;
    let visibility = &input_enum.vis;

    let variant_type_name = format_ident!("{}PackedType", enum_name);
    let variant_struct_name = format_ident!("{}Packed", enum_name);

    let mut all_variants = Vec::new();

    for variant in &input_enum.variants {
        let ty = match &variant.fields {
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => &fields.unnamed[0].ty,
            _ => {
                let msg = "dyn_variant supports only tuple-style variants with a single field, e.g., `MyPacked(String)`";
                return syn::Error::new_spanned(&variant.fields, msg)
                    .to_compile_error()
                    .into();
            }
        };

        all_variants.push(PackedInfo {
            name: &variant.ident,
            ty,
            field_name: format_ident!("__{}", variant.ident.to_string().to_lowercase()),
        });
    }

    let variant_names = all_variants.iter().map(|v| v.name);
    let variant_types = all_variants.iter().map(|v| v.ty);
    let variant_struct_field_names = all_variants.iter().map(|v| &v.field_name);

    // --- Generate `pack()` method match arms (with previous fix) ---
    let pack_match_arms = all_variants.iter().map(|variant_info| {
        let variant_name = variant_info.name;
        let active_field_name = &variant_info.field_name;

        let field_initializers = all_variants.iter().map(|current_variant| {
            let current_field_name = &current_variant.field_name;
            let current_type = current_variant.ty;

            if current_field_name == active_field_name {
                quote! { #current_field_name: Some(value).into() }
            } else {
                quote! { #current_field_name: None::<#current_type>.into() }
            }
        });

        quote! {
            #enum_name::#variant_name(value) => {
                #variant_struct_name {
                    variant: #variant_type_name::#variant_name,
                    #( #field_initializers ),*
                }
            }
        }
    });

    // --- Generate `unpack()` method match arms (with NEW fix) ---
    let unpack_match_arms = all_variants.iter().map(|variant_info| {
        let variant_name = variant_info.name;
        let struct_field_name = &variant_info.field_name;
        quote! {
            #variant_type_name::#variant_name => {
                let value = self.#struct_field_name.as_ref().unwrap_or_else(|| {
                    panic!(
                        "Invalid dynamic variant state: `{}` variant is missing its data.",
                        stringify!(#variant_name)
                    );
                });
                #enum_name::#variant_name(value.clone())
            }
        }
    });

    // --- Assemble the final token stream ---
    let expanded = quote! {
        #input_enum

        impl #enum_name {
            #[doc = "Packs the enum into its flattened struct representation."]
            #visibility fn pack(self) -> #variant_struct_name {
                match self {
                    #( #pack_match_arms ),*
                }
            }
        }

        #[derive(::zvariant::Type, Debug, ::serde::Deserialize, ::serde::Serialize, PartialEq, Eq, Clone, Copy)]
        #[allow(dead_code)]
        #visibility enum #variant_type_name {
            #( #variant_names ),*
        }

        #[derive(::zvariant::Type, ::serde::Serialize, ::serde::Deserialize, Debug)]
        #visibility struct #variant_struct_name {
            #visibility variant: #variant_type_name,
            #( #visibility #variant_struct_field_names: ::zvariant::Optional<#variant_types> ),*
        }

        impl #variant_struct_name {
            #[doc = "Unpacks the flattened struct back into the original enum."]
            #visibility fn unpack(self) -> #enum_name {
                match self.variant {
                    #( #unpack_match_arms ),*
                }
            }
        }
    };

    expanded.into()
}
