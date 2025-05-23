use crate::ast::definition::{Context, Directives, FieldsDefinition, UnionMemberTypes};
use crate::ast::{ConstDirectives, DepthLimiter, FromTokens, ParseError, Tokens, TryFromTokens};
use crate::lexical_token::{Name, PunctuatorType, StringValue};
use bluejay_core::definition::{HasDirectives, UnionTypeDefinition as CoreUnionTypeDefinition};

#[derive(Debug)]
pub struct UnionTypeDefinition<'a, C: Context> {
    description: Option<StringValue<'a>>,
    name: Name<'a>,
    directives: Option<Directives<'a, C>>,
    member_types: UnionMemberTypes<'a, C>,
    fields_definition: FieldsDefinition<'a, C>,
}

impl<'a, C: Context> CoreUnionTypeDefinition for UnionTypeDefinition<'a, C> {
    type UnionMemberTypes = UnionMemberTypes<'a, C>;
    type FieldsDefinition = FieldsDefinition<'a, C>;

    fn description(&self) -> Option<&str> {
        self.description.as_ref().map(AsRef::as_ref)
    }

    fn name(&self) -> &str {
        self.name.as_ref()
    }

    fn union_member_types(&self) -> &Self::UnionMemberTypes {
        &self.member_types
    }

    fn fields_definition(&self) -> &Self::FieldsDefinition {
        &self.fields_definition
    }
}

impl<'a, C: Context> UnionTypeDefinition<'a, C> {
    pub(crate) const UNION_IDENTIFIER: &'static str = "union";

    pub(crate) fn name(&self) -> &Name<'a> {
        &self.name
    }
}

impl<'a, C: Context> FromTokens<'a> for UnionTypeDefinition<'a, C> {
    fn from_tokens(
        tokens: &mut impl Tokens<'a>,
        depth_limiter: DepthLimiter,
    ) -> Result<Self, ParseError> {
        let description = tokens.next_if_string_value();
        tokens.expect_name_value(Self::UNION_IDENTIFIER)?;
        let name = tokens.expect_name()?;
        let directives =
            ConstDirectives::try_from_tokens(tokens, depth_limiter.bump()?).transpose()?;
        tokens.expect_punctuator(PunctuatorType::Equals)?;
        let member_types = UnionMemberTypes::from_tokens(tokens, depth_limiter.bump()?)?;
        Ok(Self {
            description,
            name,
            directives: directives.map(Directives::from),
            member_types,
            fields_definition: FieldsDefinition::__typename(),
        })
    }
}

impl<'a, C: Context> HasDirectives for UnionTypeDefinition<'a, C> {
    type Directives = Directives<'a, C>;

    fn directives(&self) -> Option<&Self::Directives> {
        self.directives.as_ref()
    }
}
