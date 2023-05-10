use crate::ast::definition::{Context, UnionMemberType};
use crate::ast::{FromTokens, ParseError, Tokens};
use crate::lexical_token::PunctuatorType;
use bluejay_core::definition::UnionMemberTypes as CoreUnionMemberTypes;
use bluejay_core::derive::AsIter;

#[derive(Debug, AsIter)]
pub struct UnionMemberTypes<'a, C: Context> {
    union_member_types: Vec<UnionMemberType<'a, C>>,
}

impl<'a, C: Context> CoreUnionMemberTypes for UnionMemberTypes<'a, C> {
    type UnionMemberType = UnionMemberType<'a, C>;
}

impl<'a, C: Context> FromTokens<'a> for UnionMemberTypes<'a, C> {
    fn from_tokens(tokens: &mut impl Tokens<'a>) -> Result<Self, ParseError> {
        tokens.next_if_punctuator(PunctuatorType::Pipe);
        let mut union_member_types = vec![UnionMemberType::from_tokens(tokens)?];
        while tokens.next_if_punctuator(PunctuatorType::Pipe).is_some() {
            union_member_types.push(UnionMemberType::from_tokens(tokens)?);
        }
        Ok(Self { union_member_types })
    }
}
