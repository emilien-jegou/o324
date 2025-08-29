use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Fields, Ident, ItemEnum, Type};

// A helper enum to distinguish between variant types.
enum VariantInfo<'a> {
    /// A variant with a single field, e.g., `Single(TaskDto)`.
    Tuple {
        name: &'a Ident,
        ty: &'a Type,
        field_name: Ident,
    },
    /// A variant with no fields, e.g., `NotFound`.
    Unit { name: &'a Ident },
}

impl<'a> VariantInfo<'a> {
    /// Helper to get the variant's name regardless of its type.
    fn name(&self) -> &'a Ident {
        match self {
            VariantInfo::Tuple { name, .. } => name,
            VariantInfo::Unit { name } => name,
        }
    }
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
        match &variant.fields {
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                let ty = &fields.unnamed[0].ty;
                all_variants.push(VariantInfo::Tuple {
                    name: &variant.ident,
                    ty,
                    field_name: format_ident!("__{}", variant.ident.to_string().to_lowercase()),
                });
            }
            Fields::Unit => {
                all_variants.push(VariantInfo::Unit {
                    name: &variant.ident,
                });
            }
            _ => {
                let msg = "dyn_variant supports only tuple-style variants with a single field (e.g., `MyVariant(String)`) or unit-style variants (e.g., `MyVariant`).";
                return syn::Error::new_spanned(&variant.fields, msg)
                    .to_compile_error()
                    .into();
            }
        };
    }

    let tuple_variants: Vec<_> = all_variants
        .iter()
        .filter(|v| matches!(v, VariantInfo::Tuple { .. }))
        .collect();

    let variant_names = all_variants.iter().map(|v| v.name());

    let variant_struct_field_names = tuple_variants.iter().map(|v| match v {
        VariantInfo::Tuple { field_name, .. } => field_name,
        _ => unreachable!(),
    });
    let variant_types = tuple_variants.iter().map(|v| match v {
        VariantInfo::Tuple { ty, .. } => ty,
        _ => unreachable!(),
    });

    // --- Generate `pack()` method match arms ---
    let pack_match_arms = all_variants.iter().map(|variant_info| {
        let variant_name = variant_info.name();

        let field_initializers = tuple_variants.iter().map(|v| {
            // `v` is always a VariantInfo::Tuple here
            let (current_field_name, current_type) = match v {
                VariantInfo::Tuple { field_name, ty, .. } => (field_name, ty),
                _ => unreachable!(),
            };

            if let VariantInfo::Tuple {
                name: active_variant_name,
                ..
            } = variant_info
            {
                // FIX #1: Dereference one side for an unambiguous comparison.
                // This resolves the subtle type inference issues that caused the trait errors.
                if *active_variant_name == v.name() {
                    quote! { #current_field_name: Some(value).into() }
                } else {
                    quote! { #current_field_name: None::<#current_type>.into() }
                }
            } else {
                // The active variant is a Unit variant, so all data fields are None.
                quote! { #current_field_name: None::<#current_type>.into() }
            }
        });

        match variant_info {
            VariantInfo::Tuple { .. } => quote! {
                #enum_name::#variant_name(value) => #variant_struct_name {
                    variant: #variant_type_name::#variant_name,
                    #( #field_initializers ),*
                }
            },
            VariantInfo::Unit { .. } => quote! {
                #enum_name::#variant_name => #variant_struct_name {
                    variant: #variant_type_name::#variant_name,
                    #( #field_initializers ),*
                }
            },
        }
    });

    let unpack_match_arms = all_variants.iter().map(|variant_info| {
        let variant_name = variant_info.name();

        match variant_info {
            VariantInfo::Tuple { field_name: struct_field_name, .. } => quote! {
                #variant_type_name::#variant_name => {
                    let value = self.#struct_field_name.take().unwrap_or_else(|| {
                        unreachable!(
                            "Internal consistency error in dyn_variant: The variant is `{}` but its corresponding data field is None. This should never happen.",
                            stringify!(#variant_name)
                        );
                    });
                    #enum_name::#variant_name(value)
                }
            },
            VariantInfo::Unit { .. } => quote! {
                #variant_type_name::#variant_name => #enum_name::#variant_name,
            },
        }
    });

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
            // FIX #2: Add `mut self` to allow calling `.take()` which requires a mutable receiver.
            #visibility fn unpack(mut self) -> #enum_name {
                match self.variant {
                    #( #unpack_match_arms ),*
                }
            }
        }
    };

    expanded.into()
}
