use crate::definition::visibility_scoped::{
    Cache, EnumTypeDefinition, InputObjectTypeDefinition, ScalarTypeDefinition, TypeDefinition,
    Warden,
};
use crate::definition::{
    self, prelude::*, BaseInputTypeReference, InputTypeReference, SchemaDefinition,
};
use crate::BuiltinScalarDefinition;

pub enum BaseInputType<'a, S: SchemaDefinition, W: Warden<SchemaDefinition = S>> {
    BuiltinScalar(BuiltinScalarDefinition),
    CustomScalar(&'a ScalarTypeDefinition<'a, S, W>),
    InputObject(&'a InputObjectTypeDefinition<'a, S, W>),
    Enum(&'a EnumTypeDefinition<'a, S, W>),
}

impl<'a, S: SchemaDefinition + 'a, W: Warden<SchemaDefinition = S>> BaseInputType<'a, S, W> {
    pub(crate) fn new(inner: &'a S::BaseInputType, cache: &'a Cache<'a, S, W>) -> Self {
        let type_definition = cache
            .get_or_create_type_definition_by_name(inner.as_ref().name())
            .unwrap();

        match type_definition {
            TypeDefinition::BuiltinScalar(bstd) => Self::BuiltinScalar(*bstd),
            TypeDefinition::CustomScalar(cstd) => Self::CustomScalar(cstd),
            TypeDefinition::Enum(etd) => Self::Enum(etd),
            TypeDefinition::InputObject(iotd) => Self::InputObject(iotd),
            TypeDefinition::Interface(_) | TypeDefinition::Object(_) | TypeDefinition::Union(_) => {
                panic!("Schema definition does not have unique type names");
            }
        }
    }
}

impl<'a, S: SchemaDefinition + 'a, W: Warden<SchemaDefinition = S>> definition::BaseInputType
    for BaseInputType<'a, S, W>
{
    type CustomScalarTypeDefinition = ScalarTypeDefinition<'a, S, W>;
    type EnumTypeDefinition = EnumTypeDefinition<'a, S, W>;
    type InputObjectTypeDefinition = InputObjectTypeDefinition<'a, S, W>;

    fn as_ref(&self) -> BaseInputTypeReference<'_, Self> {
        match self {
            Self::InputObject(iotd) => BaseInputTypeReference::InputObject(*iotd),
            Self::CustomScalar(cstd) => BaseInputTypeReference::CustomScalar(*cstd),
            Self::BuiltinScalar(bstd) => BaseInputTypeReference::BuiltinScalar(*bstd),
            Self::Enum(etd) => BaseInputTypeReference::Enum(*etd),
        }
    }
}

pub enum InputType<'a, S: SchemaDefinition, W: Warden<SchemaDefinition = S>> {
    Base(BaseInputType<'a, S, W>, bool),
    List(Box<Self>, bool),
}

impl<'a, S: SchemaDefinition + 'a, W: Warden<SchemaDefinition = S>> InputType<'a, S, W> {
    pub(crate) fn new(inner: &'a S::InputType, cache: &'a Cache<'a, S, W>) -> Self {
        match inner.as_ref() {
            InputTypeReference::Base(b, required) => {
                Self::Base(BaseInputType::new(b, cache), required)
            }
            InputTypeReference::List(inner, required) => {
                Self::List(Box::new(Self::new(inner, cache)), required)
            }
        }
    }
}

impl<'a, S: SchemaDefinition + 'a, W: Warden<SchemaDefinition = S>> definition::InputType
    for InputType<'a, S, W>
{
    type BaseInputType = BaseInputType<'a, S, W>;

    fn as_ref(&self) -> InputTypeReference<'_, Self> {
        match self {
            Self::Base(b, required) => InputTypeReference::Base(b, *required),
            Self::List(inner, required) => InputTypeReference::List(inner, *required),
        }
    }
}
