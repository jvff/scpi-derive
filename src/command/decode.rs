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
            command_decode_with_named_fields(pairs, named_fields)
        }
        Fields::Unnamed(ref unnamed_fields) => {
            command_decode_with_unnamed_fields(name, pairs, unnamed_fields)
        }
        Fields::Unit => command_decode_without_fields(pairs),
    }
}

fn command_decode_with_named_fields(
    _pairs: Pairs<Rule, StrInput>,
    _fields: &FieldsNamed,
) -> Tokens {
    quote!(unimplemented!();)
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

    let collected_fields = collect_parameters(field_index, num_fields);

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

fn command_decode_without_fields(pairs: Pairs<Rule, StrInput>) -> Tokens {
    let command_str = command_str_without_fields(pairs);

    quote! {
        if message == #command_str {
            Some(Self {})
        } else {
            None
        }
    }
}

fn build_decode_parser<'a, F, I>(
    pairs: Pairs<Rule, StrInput>,
    fields: &mut F,
    field_indices: &mut I,
) -> Tokens
where
    F: Iterator<Item = &'a Field>,
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
            Rule::space => {
                parse_steps.append_all(quote!(many1!(tag!(" ")) >>));
            }
            Rule::parameter => {
                let field_index = field_indices.next()
                    .expect("more parameters than fields in SCPI command");
                let field_data = fields.next()
                    .expect("more parameters than fields in SCPI command");

                let field_type = &field_data.ty;
                let field_name =
                    Term::intern(&format!("field_{}", field_index));

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

fn collect_parameters<I>(indices: I, last_index: usize) -> Tokens
where
    I: Iterator<Item = usize>,
{
    let mut parameters = Tokens::new();
    let mut indices = indices.peekable();

    if indices.peek() == Some(&0) {
        indices.next();
        parameters.append_all(quote!(Default::default()));
    } else {
        let field_name  = Term::intern("field_0");
        let last_collected = match indices.peek() {
            Some(index) => index.clone(),
            None => last_index,
        };

        parameters.append_all(quote!(#field_name));

        for index in 1..last_collected {
            let field_name = Term::intern(&format!("field_{}", index));

            parameters.append_all(quote!(, #field_name));
        }
    }

    for _ in indices {
        parameters.append_all(quote!(, Default::default()));
    }

    parameters
}
