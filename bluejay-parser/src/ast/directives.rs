use crate::ast::{Directive, FromTokens, IsMatch, ParseError, Tokens, TryFromTokens};
use bluejay_core::AsIter;

#[derive(Debug)]
pub struct Directives<'a, const CONST: bool>(Vec<Directive<'a, CONST>>);

pub type ConstDirectives<'a> = Directives<'a, true>;
pub type VariableDirectives<'a> = Directives<'a, false>;

impl<'a, const CONST: bool> FromTokens<'a> for Directives<'a, CONST> {
    fn from_tokens(tokens: &mut impl Tokens<'a>) -> Result<Self, ParseError> {
        let mut directives: Vec<Directive<'a, CONST>> = Vec::new();
        while let Some(directive) = Directive::try_from_tokens(tokens) {
            directives.push(directive?);
        }
        Ok(Self(directives))
    }
}

impl<'a, const CONST: bool> IsMatch<'a> for Directives<'a, CONST> {
    fn is_match(tokens: &mut impl Tokens<'a>) -> bool {
        Directive::<'a, CONST>::is_match(tokens)
    }
}

impl<'a, const CONST: bool> bluejay_core::Directives<CONST> for Directives<'a, CONST> {
    type Directive = Directive<'a, CONST>;
}

impl<'a, const CONST: bool> AsIter for Directives<'a, CONST> {
    type Item = Directive<'a, CONST>;
    type Iterator<'b> = std::slice::Iter<'b, Self::Item> where 'a: 'b;

    fn iter(&self) -> Self::Iterator<'_> {
        self.0.iter()
    }
}
