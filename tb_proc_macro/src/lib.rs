use proc_macro::TokenStream;

use quote::*;
use syn::*;

#[proc_macro_attribute]
pub fn system(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as ItemStruct);
    let output = quote! {
        #[derive(Default)]
        #item
        inventory::submit! {
            SystemInfo::new::<TestSystem>()
        }
    };
    output.into()
}

#[proc_macro_attribute]
pub fn component(attr: TokenStream, item: TokenStream) -> TokenStream {
    let storage = if attr.is_empty() {
        parse_quote!(DenseStorageItems)
    } else {
        parse_macro_input!(attr as Path)
    };
    let component_define = parse_macro_input!(item as ItemStruct);
    let component_name = &component_define.ident;
    let output = quote! {
        #component_define

        impl Component for #component_name {
            type StorageItems = #storage<Self>;
        }
    };
    output.into()
}
