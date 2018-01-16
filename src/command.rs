use pest::inputs::StrInput;
use pest::iterators::Pairs;
use pest::Parser;
use quote::Tokens;
use syn::{Fields, FieldsNamed, FieldsUnnamed};

#[cfg(debug_assertions)]
const _GRAMMAR: &'static str = include_str!("command.pest");

#[derive(Parser)]
#[grammar = "command.pest"]
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

pub fn command_decode(pairs: Pairs<Rule, StrInput>, fields: &Fields) -> Tokens {
    match *fields  {
        Fields::Named(ref named_fields) => {
            command_decode_with_named_fields(pairs, named_fields)
        }
        Fields::Unnamed(ref unnamed_fields) => {
            command_decode_with_unnamed_fields(pairs, unnamed_fields)
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
    _pairs: Pairs<Rule, StrInput>,
    _fields: &FieldsUnnamed,
) -> Tokens {
    unimplemented!();
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
    _pairs: Pairs<Rule, StrInput>,
    _fields: &FieldsUnnamed,
) -> Tokens {
    quote!(unimplemented!();)
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
