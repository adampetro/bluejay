use crate::ast::definition::{
    Context, CustomScalarTypeDefinition, EnumTypeDefinition, InterfaceTypeDefinition,
    ObjectTypeDefinition, TypeDefinition, UnionTypeDefinition,
};
use crate::ast::{FromTokens, ParseError, Tokens};
use crate::lexical_token::{Name, PunctuatorType};
use crate::{HasSpan, Span};
use bluejay_core::definition::{
    BaseOutputType as CoreBaseOutputType, BaseOutputTypeReference, OutputType as CoreOutputType,
    OutputTypeReference,
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
    Interface(Arc<InterfaceTypeDefinition<'a, C>>),
    Object(Arc<ObjectTypeDefinition<'a, C>>),
    Union(Arc<UnionTypeDefinition<'a, C>>),
}

#[derive(Debug)]
pub struct BaseOutputType<'a, C: Context + 'a> {
    name: Name<'a>,
    inner: OnceCell<BaseInner<'a, C>>,
    context: PhantomData<C>,
}

impl<'a, C: Context + 'a> CoreBaseOutputType for BaseOutputType<'a, C> {
    type CustomScalarTypeDefinition = CustomScalarTypeDefinition<'a, C>;
    type EnumTypeDefinition = EnumTypeDefinition<'a>;
    type InterfaceTypeDefinition = InterfaceTypeDefinition<'a, C>;
    type ObjectTypeDefinition = ObjectTypeDefinition<'a, C>;
    type UnionTypeDefinition = UnionTypeDefinition<'a, C>;

    fn as_ref(&self) -> BaseOutputTypeReference<'_, Self> {
        match self.inner.get().unwrap() {
            BaseInner::BuiltinScalar(bstd) => BaseOutputTypeReference::BuiltinScalar(*bstd),
            BaseInner::CustomScalar(cstd) => BaseOutputTypeReference::CustomScalar(cstd.as_ref()),
            BaseInner::Enum(etd) => BaseOutputTypeReference::Enum(etd.as_ref()),
            BaseInner::Interface(itd) => BaseOutputTypeReference::Interface(itd.as_ref()),
            BaseInner::Object(otd) => BaseOutputTypeReference::Object(otd.as_ref()),
            BaseInner::Union(utd) => BaseOutputTypeReference::Union(utd.as_ref()),
        }
    }
}

impl<'a, C: Context + 'a> BaseOutputType<'a, C> {
    fn new(name: Name<'a>) -> Self {
        Self {
            name,
            inner: OnceCell::new(),
            context: Default::default(),
        }
    }

    pub(crate) fn set_type(&self, type_definition: &TypeDefinition<'a, C>) -> Result<(), ()> {
        let inner = match type_definition {
            TypeDefinition::BuiltinScalar(bstd) => Ok(BaseInner::BuiltinScalar(*bstd)),
            TypeDefinition::CustomScalar(cstd) => Ok(BaseInner::CustomScalar(cstd.clone())),
            TypeDefinition::Enum(etd) => Ok(BaseInner::Enum(etd.clone())),
            TypeDefinition::InputObject(_) => Err(()),
            TypeDefinition::Interface(itd) => Ok(BaseInner::Interface(itd.clone())),
            TypeDefinition::Object(otd) => Ok(BaseInner::Object(otd.clone())),
            TypeDefinition::Union(utd) => Ok(BaseInner::Union(utd.clone())),
        }?;

        self.inner.set(inner).map_err(|_| ())
    }

    pub(crate) fn name(&self) -> &Name<'a> {
        &self.name
    }
}

#[derive(Debug)]
pub enum OutputType<'a, C: Context + 'a> {
    Base(BaseOutputType<'a, C>, bool, Span),
    List(Box<Self>, bool, Span),
}

impl<'a, C: Context + 'a> CoreOutputType for OutputType<'a, C> {
    type BaseOutputType = BaseOutputType<'a, C>;

    fn as_ref(&self) -> OutputTypeReference<'_, Self> {
        match self {
            Self::Base(base, required, _) => OutputTypeReference::Base(base, *required),
            Self::List(inner, required, _) => OutputTypeReference::List(inner.as_ref(), *required),
        }
    }
}

impl<'a, C: Context + 'a> FromTokens<'a> for OutputType<'a, C> {
    fn from_tokens(tokens: &mut impl Tokens<'a>) -> Result<Self, ParseError> {
        if let Some(open_span) = tokens.next_if_punctuator(PunctuatorType::OpenSquareBracket) {
            let inner = Self::from_tokens(tokens).map(Box::new)?;
            let close_span = tokens.expect_punctuator(PunctuatorType::CloseSquareBracket)?;
            let bang_span = tokens.next_if_punctuator(PunctuatorType::Bang);
            let span = open_span.merge(&close_span);
            Ok(Self::List(inner, bang_span.is_some(), span))
        } else if let Some(base_name) = tokens.next_if_name() {
            let bang_span = tokens.next_if_punctuator(PunctuatorType::Bang);
            let span = if let Some(bang_span) = &bang_span {
                base_name.span().merge(bang_span)
            } else {
                base_name.span().clone()
            };
            let base = BaseOutputType::new(base_name);
            Ok(Self::Base(base, bang_span.is_some(), span))
        } else {
            Err(tokens.unexpected_token())
        }
    }
}
