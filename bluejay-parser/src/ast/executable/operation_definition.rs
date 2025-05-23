use crate::ast::executable::{SelectionSet, VariableDefinitions};
use crate::ast::{
    DepthLimiter, FromTokens, IsMatch, OperationType, ParseError, Tokens, TryFromTokens,
    VariableDirectives,
};
use crate::lexical_token::Name;
use crate::{HasSpan, Span};
use bluejay_core::{
    executable::{OperationDefinition as CoreOperationDefinition, OperationDefinitionReference},
    Indexable,
};

#[derive(Debug)]
pub enum OperationDefinition<'a> {
    Explicit(ExplicitOperationDefinition<'a>),
    Implicit(ImplicitOperationDefinition<'a>),
}

impl<'a> OperationDefinition<'a> {
    pub fn selection_set(&self) -> &SelectionSet<'a> {
        match self {
            Self::Explicit(e) => &e.selection_set,
            Self::Implicit(i) => &i.selection_set,
        }
    }
}

impl Indexable for OperationDefinition<'_> {
    type Id = Span;

    fn id(&self) -> &Self::Id {
        self.span()
    }
}

impl<'a> CoreOperationDefinition for OperationDefinition<'a> {
    type ExplicitOperationDefinition = ExplicitOperationDefinition<'a>;
    type ImplicitOperationDefinition = ImplicitOperationDefinition<'a>;

    fn as_ref(&self) -> OperationDefinitionReference<'_, Self> {
        match self {
            Self::Explicit(e) => OperationDefinitionReference::Explicit(e),
            Self::Implicit(i) => OperationDefinitionReference::Implicit(i),
        }
    }
}

impl<'a> FromTokens<'a> for OperationDefinition<'a> {
    #[inline]
    fn from_tokens(
        tokens: &mut impl Tokens<'a>,
        depth_limiter: DepthLimiter,
    ) -> Result<Self, ParseError> {
        if let Some(operation_type) =
            OperationType::try_from_tokens(tokens, depth_limiter.bump()?).transpose()?
        {
            let name = tokens.next_if_name();
            let variable_definitions =
                VariableDefinitions::try_from_tokens(tokens, depth_limiter.bump()?).transpose()?;
            let directives =
                VariableDirectives::try_from_tokens(tokens, depth_limiter.bump()?).transpose()?;
            let selection_set = SelectionSet::from_tokens(tokens, depth_limiter.bump()?)?;
            let span = operation_type.span().merge(selection_set.span());
            Ok(Self::Explicit(ExplicitOperationDefinition {
                operation_type,
                name,
                variable_definitions,
                directives,
                selection_set,
                span,
            }))
        } else if let Some(selection_set) =
            SelectionSet::try_from_tokens(tokens, depth_limiter.bump()?).transpose()?
        {
            Ok(Self::Implicit(ImplicitOperationDefinition {
                selection_set,
            }))
        } else {
            Err(tokens.unexpected_token())
        }
    }
}

impl<'a> IsMatch<'a> for OperationDefinition<'a> {
    #[inline]
    fn is_match(tokens: &mut impl Tokens<'a>) -> bool {
        OperationType::is_match(tokens) || SelectionSet::is_match(tokens)
    }
}

impl HasSpan for OperationDefinition<'_> {
    fn span(&self) -> &Span {
        match self {
            Self::Explicit(e) => e.span(),
            Self::Implicit(i) => i.span(),
        }
    }
}

#[derive(Debug)]
pub struct ExplicitOperationDefinition<'a> {
    operation_type: OperationType,
    name: Option<Name<'a>>,
    variable_definitions: Option<VariableDefinitions<'a>>,
    directives: Option<VariableDirectives<'a>>,
    selection_set: SelectionSet<'a>,
    span: Span,
}

impl<'a> bluejay_core::executable::ExplicitOperationDefinition for ExplicitOperationDefinition<'a> {
    type VariableDefinitions = VariableDefinitions<'a>;
    type Directives = VariableDirectives<'a>;
    type SelectionSet = SelectionSet<'a>;

    fn operation_type(&self) -> bluejay_core::OperationType {
        (&self.operation_type).into()
    }

    fn name(&self) -> Option<&str> {
        self.name.as_ref().map(|name| name.as_ref())
    }

    fn variable_definitions(&self) -> Option<&Self::VariableDefinitions> {
        self.variable_definitions.as_ref()
    }

    fn directives(&self) -> Option<&Self::Directives> {
        self.directives.as_ref()
    }

    fn selection_set(&self) -> &Self::SelectionSet {
        &self.selection_set
    }
}

impl<'a> ExplicitOperationDefinition<'a> {
    pub fn name(&self) -> Option<&Name<'a>> {
        self.name.as_ref()
    }

    pub fn operation_type(&self) -> &OperationType {
        &self.operation_type
    }
}

impl HasSpan for ExplicitOperationDefinition<'_> {
    fn span(&self) -> &Span {
        &self.span
    }
}

#[derive(Debug)]
pub struct ImplicitOperationDefinition<'a> {
    selection_set: SelectionSet<'a>,
}

impl<'a> bluejay_core::executable::ImplicitOperationDefinition for ImplicitOperationDefinition<'a> {
    type SelectionSet = SelectionSet<'a>;

    fn selection_set(&self) -> &Self::SelectionSet {
        &self.selection_set
    }
}

impl HasSpan for ImplicitOperationDefinition<'_> {
    fn span(&self) -> &Span {
        self.selection_set.span()
    }
}
