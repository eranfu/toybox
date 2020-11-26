use proc_macro::TokenStream;

use quote::quote;
use syn::ItemStruct;

#[proc_macro_attribute]
pub fn system(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = syn::parse_macro_input!(item as ItemStruct);
    let output = quote! {
        #[derive(Default)]
        #item
        inventory::submit! {
            SystemInfo::new::<TestSystem>()
        }
    };
    output.into()
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
