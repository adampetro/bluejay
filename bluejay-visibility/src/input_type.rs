use crate::{
    Cache, EnumTypeDefinition, InputObjectTypeDefinition, ScalarTypeDefinition,
    SchemaDefinitionWithVisibility, TypeDefinition,
};
use bluejay_core::definition::{
    self, prelude::*, BaseInputTypeReference, InputTypeReference, TypeDefinitionReference,
};
use bluejay_core::BuiltinScalarDefinition;

pub enum BaseInputType<'a, S: SchemaDefinitionWithVisibility> {
    BuiltinScalar(BuiltinScalarDefinition),
    CustomScalar(&'a ScalarTypeDefinition<'a, S>),
    InputObject(&'a InputObjectTypeDefinition<'a, S>),
    Enum(&'a EnumTypeDefinition<'a, S>),
}

impl<'a, S: SchemaDefinitionWithVisibility + 'a> BaseInputType<'a, S> {
    pub(crate) fn new(inner: &'a S::BaseInputType, cache: &'a Cache<'a, S>) -> Option<Self> {
        let tdr = match inner.as_ref() {
            BaseInputTypeReference::BuiltinScalar(bstd) => {
                TypeDefinitionReference::BuiltinScalar(bstd)
            }
            BaseInputTypeReference::CustomScalar(cstd) => {
                TypeDefinitionReference::CustomScalar(cstd)
            }
            BaseInputTypeReference::Enum(etd) => TypeDefinitionReference::Enum(etd),
            BaseInputTypeReference::InputObject(iotd) => TypeDefinitionReference::InputObject(iotd),
        };

        cache
            .get_or_create_type_definition(tdr)
            .map(|type_definition| match type_definition {
                TypeDefinition::BuiltinScalar(bstd) => Self::BuiltinScalar(*bstd),
                TypeDefinition::CustomScalar(cstd) => Self::CustomScalar(cstd),
                TypeDefinition::Enum(etd) => Self::Enum(etd),
                TypeDefinition::InputObject(iotd) => Self::InputObject(iotd),
                TypeDefinition::Interface(_)
                | TypeDefinition::Object(_)
                | TypeDefinition::Union(_) => {
                    panic!("Schema definition does not have unique type names");
                }
            })
    }
}

impl<'a, S: SchemaDefinitionWithVisibility + 'a> definition::BaseInputType
    for BaseInputType<'a, S>
{
    type CustomScalarTypeDefinition = ScalarTypeDefinition<'a, S>;
    type EnumTypeDefinition = EnumTypeDefinition<'a, S>;
    type InputObjectTypeDefinition = InputObjectTypeDefinition<'a, S>;

    fn as_ref(&self) -> BaseInputTypeReference<'_, Self> {
        match self {
            Self::InputObject(iotd) => BaseInputTypeReference::InputObject(*iotd),
            Self::CustomScalar(cstd) => BaseInputTypeReference::CustomScalar(*cstd),
            Self::BuiltinScalar(bstd) => BaseInputTypeReference::BuiltinScalar(*bstd),
            Self::Enum(etd) => BaseInputTypeReference::Enum(*etd),
        }
    }
}

pub enum InputType<'a, S: SchemaDefinitionWithVisibility> {
    Base(BaseInputType<'a, S>, bool),
    List(Box<Self>, bool),
}

impl<'a, S: SchemaDefinitionWithVisibility + 'a> InputType<'a, S> {
    pub fn new(inner: &'a S::InputType, cache: &'a Cache<'a, S>) -> Option<Self> {
        match inner.as_ref() {
            InputTypeReference::Base(b, required) => {
                BaseInputType::new(b, cache).map(|base| Self::Base(base, required))
            }
            InputTypeReference::List(inner, required) => {
                Self::new(inner, cache).map(|inner| Self::List(Box::new(inner), required))
            }
        }
    }
}

impl<'a, S: SchemaDefinitionWithVisibility + 'a> definition::InputType for InputType<'a, S> {
    type BaseInputType = BaseInputType<'a, S>;

    fn as_ref(&self) -> InputTypeReference<'_, Self> {
        match self {
            Self::Base(b, required) => InputTypeReference::Base(b, *required),
            Self::List(inner, required) => InputTypeReference::List(inner, *required),
        }
    }
}
