use crate::ast::{
    definition::{Context, Directives},
    ConstDirectives, DepthLimiter, FromTokens, ParseError, Tokens, TryFromTokens,
};
use crate::lexical_token::{Name, PunctuatorType, StringValue};
use crate::Span;
use bluejay_core::OperationType;
use std::str::FromStr;

#[derive(Debug)]
pub struct ExplicitSchemaDefinition<'a, C: Context> {
    description: Option<StringValue<'a>>,
    schema_identifier_span: Span,
    directives: Option<Directives<'a, C>>,
    root_operation_type_definitions: Vec<RootOperationTypeDefinition<'a>>,
    root_operation_type_definitions_span: Span,
}

impl<'a, C: Context> ExplicitSchemaDefinition<'a, C> {
    pub(crate) const SCHEMA_IDENTIFIER: &'static str = "schema";

    pub(crate) fn description(&self) -> Option<&StringValue> {
        self.description.as_ref()
    }

    pub(crate) fn root_operation_type_definitions(&self) -> &[RootOperationTypeDefinition<'a>] {
        &self.root_operation_type_definitions
    }

    pub(crate) fn directives(&self) -> Option<&Directives<'a, C>> {
        self.directives.as_ref()
    }

    pub(crate) fn schema_identifier_span(&self) -> &Span {
        &self.schema_identifier_span
    }

    pub(crate) fn root_operation_type_definitions_span(&self) -> &Span {
        &self.root_operation_type_definitions_span
    }
}

impl<'a, C: Context> FromTokens<'a> for ExplicitSchemaDefinition<'a, C> {
    fn from_tokens(
        tokens: &mut impl Tokens<'a>,
        depth_limiter: DepthLimiter,
    ) -> Result<Self, ParseError> {
        let description = tokens.next_if_string_value();

        let schema_identifier_span = tokens.expect_name_value(Self::SCHEMA_IDENTIFIER)?;

        let directives =
            ConstDirectives::try_from_tokens(tokens, depth_limiter.bump()?).transpose()?;

        let mut root_operation_type_definitions = Vec::new();

        let open_span = tokens.expect_punctuator(PunctuatorType::OpenBrace)?;

        let close_span = loop {
            root_operation_type_definitions.push(RootOperationTypeDefinition::from_tokens(
                tokens,
                depth_limiter.bump()?,
            )?);
            if let Some(close_span) = tokens.next_if_punctuator(PunctuatorType::CloseBrace) {
                break close_span;
            }
        };

        let root_operation_type_definitions_span = open_span.merge(&close_span);

        Ok(Self {
            description,
            schema_identifier_span,
            directives: directives.map(Directives::from),
            root_operation_type_definitions,
            root_operation_type_definitions_span,
        })
    }
}

#[derive(Debug)]
pub struct RootOperationTypeDefinition<'a> {
    operation_type: OperationType,
    name: Name<'a>,
}

impl<'a> RootOperationTypeDefinition<'a> {
    pub(crate) fn operation_type(&self) -> OperationType {
        self.operation_type
    }

    pub(crate) fn name_token(&self) -> &Name<'a> {
        &self.name
    }

    pub(crate) fn name(&self) -> &str {
        self.name.as_ref()
    }
}

impl<'a> FromTokens<'a> for RootOperationTypeDefinition<'a> {
    fn from_tokens(tokens: &mut impl Tokens<'a>, _: DepthLimiter) -> Result<Self, ParseError> {
        let operation_type = tokens.expect_name().and_then(|name| {
            OperationType::from_str(name.as_str()).map_err(|_| ParseError::ExpectedOneOf {
                span: name.into(),
                values: OperationType::POSSIBLE_VALUES,
            })
        })?;
        tokens.expect_punctuator(PunctuatorType::Colon)?;
        let name = tokens.expect_name()?;
        Ok(Self {
            operation_type,
            name,
        })
    }
}
