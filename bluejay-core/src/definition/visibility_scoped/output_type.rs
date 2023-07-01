use crate::definition::visibility_scoped::{
    Cache, EnumTypeDefinition, InterfaceTypeDefinition, ObjectTypeDefinition, ScalarTypeDefinition,
    TypeDefinition, UnionTypeDefinition, Warden,
};
use crate::definition::{
    self, prelude::*, BaseOutputTypeReference, OutputTypeReference, SchemaDefinition,
};
use crate::BuiltinScalarDefinition;

pub enum BaseOutputType<'a, S: SchemaDefinition, W: Warden<SchemaDefinition = S>> {
    BuiltinScalar(BuiltinScalarDefinition),
    CustomScalar(&'a ScalarTypeDefinition<'a, S, W>),
    Object(&'a ObjectTypeDefinition<'a, S, W>),
    Interface(&'a InterfaceTypeDefinition<'a, S, W>),
    Enum(&'a EnumTypeDefinition<'a, S, W>),
    Union(&'a UnionTypeDefinition<'a, S, W>),
}

impl<'a, S: SchemaDefinition + 'a, W: Warden<SchemaDefinition = S>> BaseOutputType<'a, S, W> {
    pub(crate) fn new(inner: &'a S::BaseOutputType, cache: &'a Cache<'a, S, W>) -> Self {
        let type_definition = cache
            .get_or_create_type_definition_by_name(inner.as_ref().name())
            .unwrap();

        match type_definition {
            TypeDefinition::BuiltinScalar(bstd) => Self::BuiltinScalar(*bstd),
            TypeDefinition::CustomScalar(cstd) => Self::CustomScalar(cstd),
            TypeDefinition::Enum(etd) => Self::Enum(etd),
            TypeDefinition::Interface(itd) => Self::Interface(itd),
            TypeDefinition::Object(otd) => Self::Object(otd),
            TypeDefinition::Union(utd) => Self::Union(utd),
            TypeDefinition::InputObject(_) => {
                panic!("Schema definition does not have unique type names");
            }
        }
    }
}

impl<'a, S: SchemaDefinition + 'a, W: Warden<SchemaDefinition = S>> definition::BaseOutputType
    for BaseOutputType<'a, S, W>
{
    type ObjectTypeDefinition = ObjectTypeDefinition<'a, S, W>;
    type CustomScalarTypeDefinition = ScalarTypeDefinition<'a, S, W>;
    type InterfaceTypeDefinition = InterfaceTypeDefinition<'a, S, W>;
    type EnumTypeDefinition = EnumTypeDefinition<'a, S, W>;
    type UnionTypeDefinition = UnionTypeDefinition<'a, S, W>;

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

pub enum OutputType<'a, S: SchemaDefinition, W: Warden<SchemaDefinition = S>> {
    Base(BaseOutputType<'a, S, W>, bool),
    List(Box<Self>, bool),
}

impl<'a, S: SchemaDefinition + 'a, W: Warden<SchemaDefinition = S>> OutputType<'a, S, W> {
    pub(crate) fn new(inner: &'a S::OutputType, cache: &'a Cache<'a, S, W>) -> Self {
        match inner.as_ref() {
            OutputTypeReference::Base(b, required) => {
                Self::Base(BaseOutputType::new(b, cache), required)
            }
            OutputTypeReference::List(inner, required) => {
                Self::List(Box::new(Self::new(inner, cache)), required)
            }
        }
    }
}

impl<'a, S: SchemaDefinition + 'a, W: Warden<SchemaDefinition = S>> definition::OutputType
    for OutputType<'a, S, W>
{
    type BaseOutputType = BaseOutputType<'a, S, W>;

    fn as_ref(&self) -> OutputTypeReference<'_, Self> {
        match self {
            Self::Base(b, required) => OutputTypeReference::Base(b, *required),
            Self::List(inner, required) => OutputTypeReference::List(inner, *required),
        }
    }
}
