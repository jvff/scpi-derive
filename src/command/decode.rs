use std::iter::empty;

use pest::inputs::StrInput;
use pest::iterators::Pairs;
use quote::Tokens;
use syn::{Field, Fields, FieldsNamed, FieldsUnnamed, Ident};

use super::*;

pub fn command_decode(
    name: &Ident,
    pairs: Pairs<Rule, StrInput>,
    fields: &Fields,
) -> Tokens {
    match *fields  {
        Fields::Named(ref named_fields) => {
            command_decode_with_named_fields(name, pairs, named_fields)
        }
        Fields::Unnamed(ref unnamed_fields) => {
            command_decode_with_unnamed_fields(name, pairs, unnamed_fields)
        }
        Fields::Unit => command_decode_without_fields(name, pairs),
    }
}

fn command_decode_with_named_fields(
    name: &Ident,
    pairs: Pairs<Rule, StrInput>,
    fields: &FieldsNamed,
) -> Tokens {
    let num_fields = fields.named.len();
    let mut fields_iter = fields.named.iter();
    let mut field_index = 0..num_fields;

    let parse_steps =
        build_decode_parser(pairs, &mut fields_iter, &mut field_index);

    let collected_fields =
        collect_parameters(fields.named.iter(), field_index, num_fields);

    quote! {
        named!(parse_cmd(&[u8]) -> #name,
            do_parse!(
                #parse_steps
                (#name { #collected_fields })
            )
        );

        let bytes = message.as_bytes();

        if let ::nom::IResult::Done(remaining, instance) = parse_cmd(bytes) {
            if remaining.len() == 0 {
                Some(instance)
            } else {
                None
            }
        } else {
            None
        }
    }
}

fn command_decode_with_unnamed_fields(
    name: &Ident,
    pairs: Pairs<Rule, StrInput>,
    fields: &FieldsUnnamed,
) -> Tokens {
    let num_fields = fields.unnamed.len();
    let mut fields_iter = fields.unnamed.iter();
    let mut field_index = 0..num_fields;

    let parse_steps =
        build_decode_parser(pairs, &mut fields_iter, &mut field_index);

    let collected_fields =
        collect_parameters(fields.unnamed.iter(), field_index, num_fields);

    quote! {
        named!(parse_cmd(&[u8]) -> #name,
            do_parse!(
                #parse_steps
                (#name(#collected_fields))
            )
        );

        let bytes = message.as_bytes();

        if let ::nom::IResult::Done(remaining, instance) = parse_cmd(bytes) {
            if remaining.len() == 0 {
                Some(instance)
            } else {
                None
            }
        } else {
            None
        }
    }
}

fn command_decode_without_fields(
    name: &Ident,
    pairs: Pairs<Rule, StrInput>,
) -> Tokens {
    let mut fields = empty();
    let mut indices = empty();

    let parse_steps = build_decode_parser(pairs, &mut fields, &mut indices);

    quote! {
        named!(parse_cmd(&[u8]) -> #name,
            do_parse!(
                #parse_steps
                (#name)
            )
        );

        let bytes = message.as_bytes();

        if let ::nom::IResult::Done(remaining, instance) = parse_cmd(bytes) {
            if remaining.len() == 0 {
                Some(instance)
            } else {
                None
            }
        } else {
            None
        }
    }
}

fn build_decode_parser<'f, F, I>(
    pairs: Pairs<Rule, StrInput>,
    fields: &mut F,
    field_indices: &mut I,
) -> Tokens
where
    F: Iterator<Item = &'f Field>,
    I: Iterator<Item = usize>,
{
    let pairs = command_inner_pairs(pairs);
    let mut parse_steps = Tokens::new();

    for pair in pairs {
        match pair.as_rule() {
            Rule::required => {
                let literal = pair.as_str();

                parse_steps.append_all(quote!(tag!(#literal) >>));
            }
            Rule::optional => {
                let literal = pair.as_str();

                parse_steps.append_all(quote!(opt!(tag!(#literal)) >>))
            }
            Rule::space => {
                parse_steps.append_all(quote!(many1!(tag!(" ")) >>));
            }
            Rule::parameter => {
                let field_index = field_indices.next()
                    .expect("more parameters than fields in SCPI command");
                let field = fields.next()
                    .expect("more parameters than fields in SCPI command");

                let field_type = &field.ty;
                let field_name = field.ident.clone().unwrap_or_else(|| {
                    Ident::from(format!("field_{}", field_index))
                });

                parse_steps.append_all(quote! {
                    #field_name: call!(
                        <#field_type as ::scpi::ScpiParameterParser>::parse
                    ) >>
                });
            }
            _ => {
                panic!(
                    "unexpected {:?} in parsed SCPI command string",
                    pair.as_str(),
                )
            }
        }
    }

    parse_steps
}

fn collect_parameters<'f, F, I>(
    mut fields: F,
    indices: I,
    last_index: usize,
) -> Tokens
where
    F: Iterator<Item = &'f Field>,
    I: Iterator<Item = usize>,
{
    let mut parameters = Tokens::new();
    let mut indices = indices.peekable();

    let field = fields.next()
        .expect("more parameters than fields in SCPI command");

    if indices.peek() == Some(&0) {
        let field_name = field.ident
            .map(|name| quote!(#name:))
            .unwrap_or_else(|| Tokens::new());

        indices.next();
        parameters.append_all(quote!(#field_name Default::default()));
    } else {
        let field_name  = field.ident.clone().unwrap_or_else(|| {
            Ident::from("field_0")
        });

        let last_collected = match indices.peek() {
            Some(index) => index.clone(),
            None => last_index,
        };

        parameters.append_all(quote!(#field_name));

        for index in 1..last_collected {
            let field = fields.next()
                .expect("more parameters than fields in SCPI command");

            let field_name = field.ident.clone().unwrap_or_else(|| {
                Ident::from(format!("field_{}", index))
            });


            parameters.append_all(quote!(, #field_name));
        }
    }

    for _ in indices {
        let field = fields.next()
            .expect("more parameters than fields in SCPI command");

        let field_name = field.ident
            .map(|name| quote!(#name:))
            .unwrap_or_else(|| Tokens::new());

        parameters.append_all(quote!(, #field_name Default::default()));
    }

    parameters
}
