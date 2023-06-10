use crate::ast::definition::{
    Context, CustomScalarTypeDefinition, EnumTypeDefinition, InputObjectTypeDefinition,
    InterfaceTypeDefinition, ObjectTypeDefinition, UnionTypeDefinition,
};
use crate::lexical_token::Name;
use bluejay_core::definition::{TypeDefinition as CoreTypeDefinition, TypeDefinitionReference};
use bluejay_core::BuiltinScalarDefinition;
use std::sync::Arc;

#[derive(Debug)]
pub enum TypeDefinition<'a, C: Context> {
    BuiltinScalar(BuiltinScalarDefinition),
    CustomScalar(Arc<CustomScalarTypeDefinition<'a, C>>),
    Object(Arc<ObjectTypeDefinition<'a, C>>),
    InputObject(Arc<InputObjectTypeDefinition<'a, C>>),
    Enum(Arc<EnumTypeDefinition<'a>>),
    Union(Arc<UnionTypeDefinition<'a, C>>),
    Interface(Arc<InterfaceTypeDefinition<'a, C>>),
}

impl<'a, C: Context> Clone for TypeDefinition<'a, C> {
    fn clone(&self) -> Self {
        match self {
            Self::BuiltinScalar(bstd) => Self::BuiltinScalar(*bstd),
            Self::CustomScalar(cstd) => Self::CustomScalar(cstd.clone()),
            Self::Enum(etd) => Self::Enum(etd.clone()),
            Self::InputObject(iotd) => Self::InputObject(iotd.clone()),
            Self::Interface(itd) => Self::Interface(itd.clone()),
            Self::Object(otd) => Self::Object(otd.clone()),
            Self::Union(utd) => Self::Union(utd.clone()),
        }
    }
}

impl<'a, C: Context> TypeDefinition<'a, C> {
    pub(crate) fn name_token(&self) -> Option<&Name<'_>> {
        match self {
            Self::BuiltinScalar(_) => None,
            Self::CustomScalar(cstd) => Some(cstd.name()),
            Self::Enum(etd) => Some(etd.name()),
            Self::InputObject(iotd) => Some(iotd.name_token()),
            Self::Interface(itd) => Some(itd.name()),
            Self::Object(otd) => Some(otd.name()),
            Self::Union(utd) => Some(utd.name()),
        }
    }

    pub(crate) fn name(&self) -> &'a str {
        match self {
            Self::BuiltinScalar(bstd) => bstd.name(),
            Self::CustomScalar(cstd) => cstd.name().as_str(),
            Self::Enum(etd) => etd.name().as_str(),
            Self::InputObject(iotd) => iotd.name_token().as_str(),
            Self::Interface(itd) => itd.name().as_str(),
            Self::Object(otd) => otd.name().as_str(),
            Self::Union(utd) => utd.name().as_str(),
        }
    }
}

impl<'a, C: Context> CoreTypeDefinition for TypeDefinition<'a, C> {
    type CustomScalarTypeDefinition = CustomScalarTypeDefinition<'a, C>;
    type ObjectTypeDefinition = ObjectTypeDefinition<'a, C>;
    type InputObjectTypeDefinition = InputObjectTypeDefinition<'a, C>;
    type EnumTypeDefinition = EnumTypeDefinition<'a>;
    type UnionTypeDefinition = UnionTypeDefinition<'a, C>;
    type InterfaceTypeDefinition = InterfaceTypeDefinition<'a, C>;

    fn as_ref(&self) -> TypeDefinitionReference<'_, Self> {
        match self {
            Self::BuiltinScalar(bstd) => TypeDefinitionReference::BuiltinScalar(*bstd),
            Self::CustomScalar(cstd) => TypeDefinitionReference::CustomScalar(cstd.as_ref()),
            Self::Object(otd) => TypeDefinitionReference::Object(otd.as_ref()),
            Self::InputObject(iotd) => TypeDefinitionReference::InputObject(iotd.as_ref()),
            Self::Enum(etd) => TypeDefinitionReference::Enum(etd.as_ref()),
            Self::Union(utd) => TypeDefinitionReference::Union(utd.as_ref()),
            Self::Interface(itd) => TypeDefinitionReference::Interface(itd.as_ref()),
        }
    }
}

impl<'a, C: Context> From<CustomScalarTypeDefinition<'a, C>> for TypeDefinition<'a, C> {
    fn from(value: CustomScalarTypeDefinition<'a, C>) -> Self {
        Self::CustomScalar(Arc::new(value))
    }
}

impl<'a, C: Context> From<ObjectTypeDefinition<'a, C>> for TypeDefinition<'a, C> {
    fn from(value: ObjectTypeDefinition<'a, C>) -> Self {
        Self::Object(Arc::new(value))
    }
}

impl<'a, C: Context> From<InputObjectTypeDefinition<'a, C>> for TypeDefinition<'a, C> {
    fn from(value: InputObjectTypeDefinition<'a, C>) -> Self {
        Self::InputObject(Arc::new(value))
    }
}

impl<'a, C: Context> From<InterfaceTypeDefinition<'a, C>> for TypeDefinition<'a, C> {
    fn from(value: InterfaceTypeDefinition<'a, C>) -> Self {
        Self::Interface(Arc::new(value))
    }
}

impl<'a, C: Context> From<EnumTypeDefinition<'a>> for TypeDefinition<'a, C> {
    fn from(value: EnumTypeDefinition<'a>) -> Self {
        Self::Enum(Arc::new(value))
    }
}

impl<'a, C: Context> From<UnionTypeDefinition<'a, C>> for TypeDefinition<'a, C> {
    fn from(value: UnionTypeDefinition<'a, C>) -> Self {
        Self::Union(Arc::new(value))
    }
}
