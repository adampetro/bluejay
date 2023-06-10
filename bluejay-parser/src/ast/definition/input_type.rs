use crate::ast::definition::{
    Context, CustomScalarTypeDefinition, DefaultContext, EnumTypeDefinition,
    InputObjectTypeDefinition, TypeDefinition,
};
use crate::ast::{FromTokens, ParseError, Tokens};
use crate::lexical_token::{Name, PunctuatorType};
use crate::{HasSpan, Span};
use bluejay_core::definition::{
    BaseInputType as CoreBaseInputType, BaseInputTypeReference, InputType as CoreInputType,
    InputTypeReference,
};
use bluejay_core::BuiltinScalarDefinition;
use once_cell::sync::OnceCell;
use std::marker::PhantomData;
use std::sync::Arc;

#[derive(Debug)]
enum BaseInner<'a, C: Context + 'a> {
    BuiltinScalar(BuiltinScalarDefinition),
    CustomScalar(Arc<CustomScalarTypeDefinition<'a, C>>),
    Enum(Arc<EnumTypeDefinition<'a>>),
    InputObject(Arc<InputObjectTypeDefinition<'a, C>>),
}

#[derive(Debug)]
pub struct BaseInputType<'a, C: Context + 'a> {
    name: Name<'a>,
    inner: OnceCell<BaseInner<'a, C>>,
    context: PhantomData<C>,
}

impl<'a, C: Context + 'a> CoreBaseInputType for BaseInputType<'a, C> {
    type CustomScalarTypeDefinition = CustomScalarTypeDefinition<'a, C>;
    type EnumTypeDefinition = EnumTypeDefinition<'a>;
    type InputObjectTypeDefinition = InputObjectTypeDefinition<'a, C>;

    fn as_ref(&self) -> BaseInputTypeReference<'_, Self> {
        match self.inner.get().unwrap() {
            BaseInner::BuiltinScalar(bstd) => BaseInputTypeReference::BuiltinScalar(*bstd),
            BaseInner::CustomScalar(cstd) => BaseInputTypeReference::CustomScalar(cstd.as_ref()),
            BaseInner::Enum(etd) => BaseInputTypeReference::Enum(etd.as_ref()),
            BaseInner::InputObject(iotd) => BaseInputTypeReference::InputObject(iotd.as_ref()),
        }
    }
}

impl<'a, C: Context + 'a> BaseInputType<'a, C> {
    pub(crate) fn name(&self) -> &Name<'a> {
        &self.name
    }

    pub(crate) fn set_type(&self, type_definition: &TypeDefinition<'a, C>) -> Result<(), ()> {
        let inner = match type_definition {
            TypeDefinition::BuiltinScalar(bstd) => Ok(BaseInner::BuiltinScalar(*bstd)),
            TypeDefinition::CustomScalar(cstd) => Ok(BaseInner::CustomScalar(cstd.clone())),
            TypeDefinition::Enum(etd) => Ok(BaseInner::Enum(etd.clone())),
            TypeDefinition::InputObject(iotd) => Ok(BaseInner::InputObject(iotd.clone())),
            TypeDefinition::Interface(_) | TypeDefinition::Object(_) | TypeDefinition::Union(_) => {
                Err(())
            }
        }?;

        self.inner.set(inner).map_err(|_| ())
    }
}

#[derive(Debug)]
pub enum InputType<'a, C: Context = DefaultContext> {
    Base(BaseInputType<'a, C>, bool, Span),
    List(Box<Self>, bool, Span),
}

impl<'a, C: Context + 'a> CoreInputType for InputType<'a, C> {
    type BaseInputType = BaseInputType<'a, C>;

    fn as_ref(&self) -> InputTypeReference<'_, Self> {
        match self {
            Self::Base(base, required, _) => InputTypeReference::Base(base, *required),
            Self::List(inner, required, _) => InputTypeReference::List(inner.as_ref(), *required),
        }
    }
}

impl<'a, C: Context + 'a> FromTokens<'a> for InputType<'a, C> {
    fn from_tokens(tokens: &mut impl Tokens<'a>) -> Result<Self, ParseError> {
        if let Some(open_span) = tokens.next_if_punctuator(PunctuatorType::OpenSquareBracket) {
            let inner = Self::from_tokens(tokens).map(Box::new)?;
            let close_span = tokens.expect_punctuator(PunctuatorType::CloseSquareBracket)?;
            let bang_span = tokens.next_if_punctuator(PunctuatorType::Bang);
            let span = open_span.merge(&close_span);
            Ok(InputType::List(inner, bang_span.is_some(), span))
        } else if let Some(base_name) = tokens.next_if_name() {
            let bang_span = tokens.next_if_punctuator(PunctuatorType::Bang);
            let span = if let Some(bang_span) = &bang_span {
                base_name.span().merge(bang_span)
            } else {
                base_name.span().clone()
            };
            let base = BaseInputType {
                name: base_name,
                inner: OnceCell::new(),
                context: Default::default(),
            };
            Ok(InputType::Base(base, bang_span.is_some(), span))
        } else {
            Err(tokens.unexpected_token())
        }
    }
}

impl<'a, C: Context> HasSpan for InputType<'a, C> {
    fn span(&self) -> &Span {
        match self {
            Self::Base(_, _, span) => span,
            Self::List(_, _, span) => span,
        }
    }
}
