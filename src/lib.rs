extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate syn;

mod scpi_attributes;

use std::iter::repeat;

use proc_macro::TokenStream;
use quote::Tokens;
use syn::{Data, DataEnum, DeriveInput};

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
        Data::Enum(ref data) => {
            implement_scpi_request_for_enum(syntax_tree, data)
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

fn implement_scpi_request_for_enum(
    syntax_tree: &DeriveInput,
    data: &DataEnum,
) -> Tokens {
    let name = syntax_tree.ident;
    let attributes = ScpiAttributes::from(syntax_tree.attrs.iter());

    let variant_names = data.variants.iter().map(|variant| variant.ident);
    let variant_names1 = variant_names.collect::<Vec<_>>();
    let variant_names2 = variant_names1.clone();

    let commands = data.variants.iter().map(|variant| {
        attributes
            .clone()
            .apply(&variant.attrs)
            .command
            .unwrap_or_else(|| {
                panic!(
                    "missing SCPI command for enum variant {}",
                    variant.ident,
                )
            })
    });
    let commands1 = commands.collect::<Vec<_>>();
    let commands2 = commands1.clone();

    let names1 = repeat(name);
    let names2 = names1.clone();

    quote! {
        impl ::std::fmt::Display for #name {
            fn fmt(
                &self,
                formatter: &mut ::std::fmt::Formatter,
            ) -> ::std::fmt::Result {
                match *self {
                    #(
                        #names1::#variant_names1 => {
                            write!(formatter, #commands1)
                        }
                    )*
                }
            }
        }

        impl ::scpi::ScpiRequest for #name {
            fn decode(message: &str) -> Option<Self> {
                #(
                    if message == #commands2 {
                        Some(#names2::#variant_names2)
                    } else
                )* {
                    None
                }
            }
        }
    }
}
