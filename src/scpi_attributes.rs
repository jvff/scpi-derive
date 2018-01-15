use quote::ToTokens;
use syn::{Attribute, Ident, Lit, Meta, MetaList, MetaNameValue, NestedMeta};

#[derive(Clone, Default)]
pub struct ScpiAttributes {
    pub command: Option<String>,
}

impl ScpiAttributes {
    pub fn apply<'a, T>(mut self, attributes: T) -> Self
    where
        T: IntoIterator<Item = &'a Attribute>,
    {
        for attribute in attributes {
            let segments = &attribute.path.segments;

            if segments.len() == 1 && segments[0].ident == "scpi" {
                self.apply_attribute(attribute);
            }
        }

        self
    }

    fn apply_attribute(&mut self, attribute: &Attribute) {
        match attribute.interpret_meta() {
            Some(Meta::List(meta_list)) => self.apply_attribute_list(meta_list),
            Some(invalid_attribute) => {
                panic!(
                    "invalid SCPI attribute: {}",
                    invalid_attribute.into_tokens(),
                )
            }
            None => panic!("invalid SCPI attribute: #[scpi]"),
        }
    }

    fn apply_attribute_list(&mut self, meta_list: MetaList) {
        for item in meta_list.nested {
            match item {
                NestedMeta::Meta(item) => self.apply_attribute_item(item),
                NestedMeta::Literal(literal) => {
                    panic!(
                        "Invalid SCPI attribute #[scpi({})]",
                        literal.into_tokens(),
                    )
                }
            }
        }
    }

    fn apply_attribute_item(&mut self, meta_item: Meta) {
        match meta_item {
            Meta::NameValue(name_value) => {
                self.apply_name_value_attribute(name_value)
            }
            invalid_attribute => {
                panic!(
                    "Invalid SCPI attribute #[scpi({})]",
                    invalid_attribute.into_tokens(),
                )
            }
        }
    }

    fn apply_name_value_attribute(&mut self, name_value: MetaNameValue) {
        if name_value.ident == Ident::from("command") {
            self.apply_command_attribute(name_value.lit)
        } else {
            panic!("invalid SCPI attribute #[scpi({} = ...)]", name_value.ident)
        }
    }

    fn apply_command_attribute(&mut self, value: Lit) {
        match value {
            Lit::Str(str_value) => {
                self.command = Some(str_value.value())
            }
            invalid_literal => {
                panic!(
                    "invalid value for SCPI command: {}",
                    invalid_literal.into_tokens(),
                )
            }
        }
    }
}

impl<'a, T> From<T> for ScpiAttributes
where
    T: IntoIterator<Item = &'a Attribute>,
{
    fn from(attributes: T) -> Self {
        ScpiAttributes::default().apply(attributes)
    }
}
