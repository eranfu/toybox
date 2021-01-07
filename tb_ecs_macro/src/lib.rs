use proc_macro::TokenStream;

use quote::*;
use syn::*;

#[proc_macro_attribute]
pub fn system(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let system_struct = parse_macro_input!(item as ItemStruct);
    let system_name = &system_struct.ident;
    let output = quote! {
        #[derive(Default)]
        #system_struct
        inventory::submit! {
            SystemInfo::new::<#system_name>()
        }
    };
    output.into()
}

#[proc_macro_attribute]
pub fn component(attr: TokenStream, item: TokenStream) -> TokenStream {
    let storage = if attr.is_empty() {
        parse_quote!(DenseStorageItems)
    } else {
        parse_macro_input!(attr as TypePath)
    };

    let component_struct = parse_macro_input!(item as ItemStruct);
    let fields: Vec<&Field> = component_struct
        .fields
        .iter()
        .filter(|field| field.ty.to_token_stream().to_string() == "Entity")
        .collect();

    let component_name = &component_struct.ident;

    let impl_component_with_entity_ref = {
        if fields.is_empty() {
            quote! {}
        } else {
            let (ref_type, ref_value): (proc_macro2::TokenStream, proc_macro2::TokenStream) = {
                if fields.len() == 1 {
                    let field_name = fields[0].ident.as_ref().unwrap();
                    (
                        "&'e mut Entity".parse().unwrap(),
                        quote! {&mut self.#field_name},
                    )
                } else {
                    let mut ref_type_str = String::from("(&'e mut Entity");
                    let mut ref_value_str = format!(
                        "(&mut self.{}",
                        fields[0].ident.as_ref().unwrap().to_string()
                    );
                    for field in fields.iter().skip(1) {
                        ref_type_str += ", &'e mut Entity";
                        ref_value_str +=
                            &format!(", &mut self.{}", field.ident.as_ref().unwrap().to_string());
                    }
                    ref_type_str += ")";
                    ref_value_str += ")";

                    (
                        ref_type_str.parse().unwrap(),
                        ref_value_str.parse().unwrap(),
                    )
                }
            };
            quote! {
                impl<'e> ComponentWithEntityRef<'e> for #component_name {
                    type Ref = #ref_type;

                    fn get_entity_ref(&'e mut self) -> Self::Ref {
                        #ref_value
                    }
                }
            }
        }
    };

    let output = quote! {
        #[derive(Clone)]
        #component_struct

        impl Component for #component_name {
            type StorageItems = #storage<Self>;
        }

        #impl_component_with_entity_ref
    };

    output.into()
}
