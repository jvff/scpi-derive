use pest::inputs::StrInput;
use pest::iterators::Pairs;
use pest::Parser;
use proc_macro2::{Literal, Term};
use quote::Tokens;
use syn::{Fields, FieldsNamed, FieldsUnnamed, Ident};

#[cfg(debug_assertions)]
const _GRAMMAR: &'static str = include_str!("command.pest");

#[derive(Parser)]
#[grammar = "command/command.pest"]
struct CommandParser;

pub fn parse_command(command_str: &str) -> Pairs<Rule, StrInput> {
    CommandParser::parse_str(Rule::command, command_str)
        .unwrap_or_else(|error| {
            panic!("invalid command syntax in {:?}: {}", command_str, error)
        })
}

pub fn command_display(pairs: Pairs<Rule, StrInput>, fields: &Fields) -> Tokens {
    match *fields {
        Fields::Named(ref named_fields) => {
            command_display_with_named_fields(pairs, named_fields)
        }
        Fields::Unnamed(ref unnamed_fields) => {
            command_display_with_unnamed_fields(pairs, unnamed_fields)
        }
        Fields::Unit => command_display_without_fields(pairs),
    }
}

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

fn command_display_with_named_fields(
    _pairs: Pairs<Rule, StrInput>,
    _fields: &FieldsNamed,
) -> Tokens {
    unimplemented!();
}

fn command_display_with_unnamed_fields(
    pairs: Pairs<Rule, StrInput>,
    fields: &FieldsUnnamed,
) -> Tokens {
    let pairs = command_inner_pairs(pairs);
    let num_fields = fields.unnamed.len();
    let mut field_index = 0;
    let mut command_str = String::new();
    let mut parameters = Tokens::new();

    for pair in pairs {
        match pair.as_rule() {
            Rule::required => command_str.push_str(pair.as_str()),
            Rule::space => command_str.push(' '),
            Rule::parameter => {
                let field_index_token = Literal::integer(field_index as i64);

                command_str.push_str("{}");
                parameters.append_all(quote!(, self.#field_index_token));

                field_index += 1;

                if field_index > num_fields {
                    panic!("more parameters than fields in SCPI command");
                }
            }
            _ => {
                panic!(
                    "unexpected {:?} in parsed SCPI command string",
                    pair.as_str(),
                )
            }
        }
    }

    quote! {
        write!(formatter, #command_str #parameters)
    }
}

fn command_display_without_fields(pairs: Pairs<Rule, StrInput>) -> Tokens {
    let command_str = command_str_without_fields(pairs);

    quote! {
        write!(formatter, #command_str)
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
    let pairs = command_inner_pairs(pairs);
    let num_fields = fields.unnamed.len();
    let mut fields_iter = fields.unnamed.iter();
    let mut field_index = 0;
    let mut parse_steps = Tokens::new();
    let mut collected_fields = Tokens::new();

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
                let field_data = fields_iter.next()
                    .expect("more parameters than fields in SCPI command");
                let field_type = &field_data.ty;
                let field_name =
                    Term::intern(&format!("field_{}", field_index));

                field_index += 1;

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

    if field_index == 0 {
        collected_fields.append_all(quote!(Default::default()));

        for _ in 1..num_fields {
            collected_fields.append_all(quote!(, Default::default()));
        }
    } else {
        let field_name  = Term::intern("field_0");

        collected_fields.append_all(quote!(#field_name));

        for index in 1..field_index {
            let field_name = Term::intern(&format!("field_{}", index));

            collected_fields.append_all(quote!(, #field_name));
        }

        for _ in field_index..num_fields {
            collected_fields.append_all(quote!(, Default::default()));
        }
    }

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

fn command_str_without_fields(pairs: Pairs<Rule, StrInput>) -> String {
    let pairs = command_inner_pairs(pairs);
    let mut command_str = String::new();

    for pair in pairs {
        match pair.as_rule() {
            Rule::required => command_str.push_str(pair.as_str()),
            Rule::space => command_str.push(' '),
            Rule::parameter => {
                panic!(
                    "types without fields can't have parameters in SCPI command"
                )
            }
            _ => {
                panic!(
                    "unexpected {:?} in parsed SCPI command string",
                    pair.as_str(),
                )
            }
        }
    }

    command_str
}

fn command_inner_pairs(
    mut pairs: Pairs<Rule, StrInput>,
) -> Pairs<Rule, StrInput> {
    let command_pair = pairs.next().expect("failed to parse SCPI command");

    if command_pair.as_rule() != Rule::command {
        panic!("failed to parse SCPI command");
    }

    command_pair.into_inner()
}
