mod decode;
mod display;

use pest::inputs::StrInput;
use pest::iterators::Pairs;
use pest::Parser;
use proc_macro2::Literal;

pub use self::decode::command_decode;
pub use self::display::command_display;

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

fn command_inner_pairs(
    mut pairs: Pairs<Rule, StrInput>,
) -> Pairs<Rule, StrInput> {
    let command_pair = pairs.next().expect("failed to parse SCPI command");

    if command_pair.as_rule() != Rule::command {
        panic!("failed to parse SCPI command");
    }

    command_pair.into_inner()
}
