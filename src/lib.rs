extern crate pest;
#[macro_use]
extern crate pest_derive;
extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate syn;

mod command;
mod scpi_attributes;

use std::iter::repeat;

use proc_macro::TokenStream;
use quote::Tokens;
use syn::{Data, DataEnum, DataStruct, DeriveInput};

use command::{command_decode, command_display, parse_command};
use scpi_attributes::ScpiAttributes;

#[proc_macro_derive(ScpiRequest, attributes(scpi))]
pub fn derive_scpi_request(input: TokenStream) -> TokenStream {
    let syntax_tree = syn::parse(input).expect("failed to parse input");

    let output = implement_scpi_request(&syntax_tree);

    output.into()
}

fn implement_scpi_request(syntax_tree: &DeriveInput) -> Tokens {
    match syntax_tree.data {
        Data::Struct(ref data) => {
            implement_scpi_request_for_struct(syntax_tree, data)
        }
        Data::Enum(ref data) => {
            implement_scpi_request_for_enum(syntax_tree, data)
        }
        Data::Union(_) => {
            panic!("deriving ScpiRequest for unions is currently not supported")
        }
    }
}

fn implement_scpi_request_for_struct(
    syntax_tree: &DeriveInput,
    data: &DataStruct,
) -> Tokens {
    let name = syntax_tree.ident;
    let attributes = ScpiAttributes::from(syntax_tree.attrs.iter());
    let command =
        attributes.command.expect("struct has no SCPI command specified");
    let parsed_command = parse_command(&command);

    let display = command_display(parsed_command.clone(), &data.fields);
    let decode = command_decode(parsed_command, &data.fields);

    quote! {
        impl ::std::fmt::Display for #name {
            fn fmt(
                &self,
                formatter: &mut ::std::fmt::Formatter,
            ) -> ::std::fmt::Result {
                #display
            }
        }

        impl ::scpi::ScpiRequest for #name {
            fn decode(message: &str) -> Option<Self> {
                #decode
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

    let commands1 = get_variant_commands(&attributes, data);
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

fn get_variant_commands(
    attributes: &ScpiAttributes,
    enum_data: &DataEnum,
) -> Vec<String> {
    enum_data.variants.iter().map(|variant| {
        let variant_attributes = attributes.clone().apply(&variant.attrs);

        match variant_attributes.command {
            Some(command) => command,
            None => {
                panic!(
                    "missing SCPI command for enum variant {}",
                    variant.ident,
                )
            }
        }
    }).collect()
}
