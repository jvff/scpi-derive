use pest::inputs::StrInput;
use pest::iterators::Pairs;
use quote::Tokens;
use syn::{Fields, FieldsNamed, FieldsUnnamed};

use super::*;

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
