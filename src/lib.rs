use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Lerp)]
pub fn derive_lerp(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = &input.ident;
    TokenStream::from(quote! {
        impl Lerp for #ident {
            type Output = Self;
            fn lerp(&self, other: &Self, t: T32) -> Self::Output {
                Self(self.0.lerp(&other.0, t))
            }
        }
    })
}
