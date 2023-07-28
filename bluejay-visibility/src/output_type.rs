use crate::{
    Cache, EnumTypeDefinition, InterfaceTypeDefinition, ObjectTypeDefinition, ScalarTypeDefinition,
    SchemaDefinitionWithVisibility, TypeDefinition, UnionTypeDefinition,
};
use bluejay_core::definition::{
    self, prelude::*, BaseOutputTypeReference, OutputTypeReference, TypeDefinitionReference,
};
use bluejay_core::BuiltinScalarDefinition;

pub enum BaseOutputType<'a, S: SchemaDefinitionWithVisibility> {
    BuiltinScalar(BuiltinScalarDefinition),
    CustomScalar(&'a ScalarTypeDefinition<'a, S>),
    Object(&'a ObjectTypeDefinition<'a, S>),
    Interface(&'a InterfaceTypeDefinition<'a, S>),
    Enum(&'a EnumTypeDefinition<'a, S>),
    Union(&'a UnionTypeDefinition<'a, S>),
}

impl<'a, S: SchemaDefinitionWithVisibility + 'a> BaseOutputType<'a, S> {
    pub(crate) fn new(inner: &'a S::BaseOutputType, cache: &'a Cache<'a, S>) -> Option<Self> {
        let tdr = match inner.as_ref() {
            BaseOutputTypeReference::BuiltinScalar(bstd) => {
                TypeDefinitionReference::BuiltinScalar(bstd)
            }
            BaseOutputTypeReference::CustomScalar(cstd) => {
                TypeDefinitionReference::CustomScalar(cstd)
            }
            BaseOutputTypeReference::Enum(etd) => TypeDefinitionReference::Enum(etd),
            BaseOutputTypeReference::Interface(itd) => TypeDefinitionReference::Interface(itd),
            BaseOutputTypeReference::Object(otd) => TypeDefinitionReference::Object(otd),
            BaseOutputTypeReference::Union(utd) => TypeDefinitionReference::Union(utd),
        };

        cache
            .get_or_create_type_definition(tdr)
            .map(|type_definition| match type_definition {
                TypeDefinition::BuiltinScalar(bstd) => Self::BuiltinScalar(*bstd),
                TypeDefinition::CustomScalar(cstd) => Self::CustomScalar(cstd),
                TypeDefinition::Enum(etd) => Self::Enum(etd),
                TypeDefinition::Interface(itd) => Self::Interface(itd),
                TypeDefinition::Object(otd) => Self::Object(otd),
                TypeDefinition::Union(utd) => Self::Union(utd),
                TypeDefinition::InputObject(_) => {
                    panic!("Schema definition does not have unique type names");
                }
            })
    }
}

impl<'a, S: SchemaDefinitionWithVisibility + 'a> definition::BaseOutputType
    for BaseOutputType<'a, S>
{
    type ObjectTypeDefinition = ObjectTypeDefinition<'a, S>;
    type CustomScalarTypeDefinition = ScalarTypeDefinition<'a, S>;
    type InterfaceTypeDefinition = InterfaceTypeDefinition<'a, S>;
    type EnumTypeDefinition = EnumTypeDefinition<'a, S>;
    type UnionTypeDefinition = UnionTypeDefinition<'a, S>;

    fn as_ref(&self) -> BaseOutputTypeReference<'_, Self> {
        match self {
            Self::Object(otd) => BaseOutputTypeReference::Object(*otd),
            Self::Interface(itd) => BaseOutputTypeReference::Interface(*itd),
            Self::CustomScalar(cstd) => BaseOutputTypeReference::CustomScalar(*cstd),
            Self::BuiltinScalar(bstd) => BaseOutputTypeReference::BuiltinScalar(*bstd),
            Self::Enum(etd) => BaseOutputTypeReference::Enum(*etd),
            Self::Union(utd) => BaseOutputTypeReference::Union(*utd),
        }
    }
}

pub enum OutputType<'a, S: SchemaDefinitionWithVisibility> {
    Base(BaseOutputType<'a, S>, bool),
    List(Box<Self>, bool),
}

impl<'a, S: SchemaDefinitionWithVisibility + 'a> OutputType<'a, S> {
    pub(crate) fn new(inner: &'a S::OutputType, cache: &'a Cache<'a, S>) -> Option<Self> {
        match inner.as_ref() {
            OutputTypeReference::Base(b, required) => {
                BaseOutputType::new(b, cache).map(|base| Self::Base(base, required))
            }
            OutputTypeReference::List(inner, required) => {
                Self::new(inner, cache).map(|inner| Self::List(Box::new(inner), required))
            }
        }
    }
}

impl<'a, S: SchemaDefinitionWithVisibility + 'a> definition::OutputType for OutputType<'a, S> {
    type BaseOutputType = BaseOutputType<'a, S>;

    fn as_ref(&self) -> OutputTypeReference<'_, Self> {
        match self {
            Self::Base(b, required) => OutputTypeReference::Base(b, *required),
            Self::List(inner, required) => OutputTypeReference::List(inner, *required),
        }
    }
}
