use crate::ast::definition::InputValueDefinition;
use crate::ast::{FromTokens, ParseError, Tokens};
use crate::lexical_token::PunctuatorType;
use crate::Span;
use bluejay_core::definition::InputFieldsDefinition as CoreInputFieldsDefinition;
use bluejay_core::AsIter;

#[derive(Debug)]
pub struct InputFieldsDefinition<'a> {
    input_field_definitions: Vec<InputValueDefinition<'a>>,
    _span: Span,
}

impl<'a> AsIter for InputFieldsDefinition<'a> {
    type Item = InputValueDefinition<'a>;
    type Iterator<'b> = std::slice::Iter<'b, Self::Item> where 'a: 'b;

    fn iter(&self) -> Self::Iterator<'_> {
        self.input_field_definitions.iter()
    }
}

impl<'a> CoreInputFieldsDefinition for InputFieldsDefinition<'a> {
    type InputValueDefinition = InputValueDefinition<'a>;
}

impl<'a> FromTokens<'a> for InputFieldsDefinition<'a> {
    fn from_tokens(tokens: &mut impl Tokens<'a>) -> Result<Self, ParseError> {
        let open_span = tokens.expect_punctuator(PunctuatorType::OpenBrace)?;
        let mut input_field_definitions: Vec<InputValueDefinition> = Vec::new();
        let close_span = loop {
            if let Some(close_span) = tokens.next_if_punctuator(PunctuatorType::CloseBrace) {
                break close_span;
            }
            input_field_definitions.push(InputValueDefinition::from_tokens(tokens)?);
        };
        let span = open_span.merge(&close_span);
        Ok(Self {
            input_field_definitions,
            _span: span,
        })
    }
}