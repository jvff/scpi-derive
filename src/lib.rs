extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate syn;

mod scpi_attributes;

use proc_macro::TokenStream;
use quote::Tokens;
use syn::{Data, DeriveInput};

use scpi_attributes::ScpiAttributes;

#[proc_macro_derive(ScpiRequest, attributes(scpi))]
pub fn derive_scpi_request(input: TokenStream) -> TokenStream {
    let syntax_tree = syn::parse(input).expect("failed to parse input");

    let output = implement_scpi_request(&syntax_tree);

    output.into()
}

fn implement_scpi_request(syntax_tree: &DeriveInput) -> Tokens {
    match syntax_tree.data {
        Data::Struct(_) => {
            implement_scpi_request_for_struct(syntax_tree)
        }
        Data::Enum(_) => {
            panic!("deriving ScpiRequest for enums is currently not supported")
        }
        Data::Union(_) => {
            panic!("deriving ScpiRequest for unions is currently not supported")
        }
    }
}

fn implement_scpi_request_for_struct(syntax_tree: &DeriveInput) -> Tokens {
    let name = syntax_tree.ident;
    let attributes = ScpiAttributes::from(syntax_tree.attrs.iter());
    let command =
        attributes.command.expect("struct has no SCPI command specified");

    quote! {
        impl ::std::fmt::Display for #name {
            fn fmt(
                &self,
                formatter: &mut ::std::fmt::Formatter,
            ) -> ::std::fmt::Result {
                write!(formatter, #command)
            }
        }

        impl ::scpi::ScpiRequest for #name {
            fn decode(message: &str) -> Option<Self> {
                if message == #command {
                    Some(#name)
                } else {
                    None
                }
            }
        }
    }
}
