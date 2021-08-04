#![feature(exact_size_is_empty)]

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
pub fn component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let component_struct = parse_macro_input!(item as ItemStruct);
    let fields: Vec<&Field> = component_struct
        .fields
        .iter()
        .filter(|field| {
            let ty = field.ty.to_token_stream().to_string();
            ty == "Entity" || ty == "Vec<Entity>"
        })
        .collect();

    let component_name = &component_struct.ident;

    let impl_component_with_entity_ref = {
        let mut fields = fields.iter();
        match fields.next() {
            None => {
                quote! {}
            }
            Some(&first) => {
                let (ref_ident, ref_type): (proc_macro2::TokenStream, proc_macro2::TokenStream) = {
                    let (first_ref_ident, first_ref_type) = {
                        let ident = &first.ident;
                        let ty = &first.ty;
                        (quote! {&mut self.#ident}, quote! {&'e mut #ty})
                    };

                    if !fields.is_empty() {
                        let (mut ref_ident, mut ref_type) = (
                            format!("({}", first_ref_ident),
                            format!("({}", first_ref_type),
                        );

                        for &field in fields {
                            ref_type +=
                                &format!(", &'e mut {}", field.ty.to_token_stream().to_string());
                            ref_ident += &format!(
                                ", &mut self.{}",
                                field.ident.to_token_stream().to_string()
                            );
                        }
                        ref_ident += ")";
                        ref_type += ")";

                        (ref_ident.parse().unwrap(), ref_type.parse().unwrap())
                    } else {
                        (first_ref_ident, first_ref_type)
                    }
                };
                quote! {
                    impl<'e> ComponentWithEntityRef<'e> for #component_name {
                        type Ref = #ref_type;

                        fn mut_entity_ref(&'e mut self) -> Self::Ref {
                            #ref_ident
                        }
                    }
                }
            }
        }
    };

    let output = quote! {
        #[derive(Clone, Deserialize, Serialize)]
        #component_struct

        impl Component for #component_name {}

        #impl_component_with_entity_ref

        inventory::submit! {
            ComponentInfo::new::<#component_name>()
        }
    };

    output.into()
}
