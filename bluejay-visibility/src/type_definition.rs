use crate::{
    Cache, EnumTypeDefinition, InputObjectTypeDefinition, InterfaceTypeDefinition,
    ObjectTypeDefinition, ScalarTypeDefinition, SchemaDefinitionWithVisibility,
    UnionTypeDefinition, Warden,
};
use bluejay_core::definition::{self, prelude::*, TypeDefinitionReference};
use bluejay_core::{AsIter, BuiltinScalarDefinition};
use enum_as_inner::EnumAsInner;

#[derive(EnumAsInner)]
pub enum TypeDefinition<'a, S: SchemaDefinitionWithVisibility> {
    BuiltinScalar(BuiltinScalarDefinition),
    CustomScalar(ScalarTypeDefinition<'a, S>),
    Object(ObjectTypeDefinition<'a, S>),
    Interface(InterfaceTypeDefinition<'a, S>),
    InputObject(InputObjectTypeDefinition<'a, S>),
    Enum(EnumTypeDefinition<'a, S>),
    Union(UnionTypeDefinition<'a, S>),
}

impl<'a, S: SchemaDefinitionWithVisibility + 'a> TypeDefinition<'a, S> {
    pub(crate) fn new(
        type_definition: TypeDefinitionReference<'a, S::TypeDefinition>,
        cache: &'a Cache<'a, S>,
    ) -> Option<Self> {
        let warden = cache.warden();
        match type_definition {
            TypeDefinitionReference::BuiltinScalar(bstd) => Some(Self::BuiltinScalar(bstd)),
            TypeDefinitionReference::CustomScalar(cstd) => warden
                .is_custom_scalar_type_definition_visible(cstd)
                .then(|| Self::CustomScalar(ScalarTypeDefinition::new(cstd, cache))),
            TypeDefinitionReference::Object(otd) => warden
                .is_object_type_definition_visible(otd)
                .then(|| Self::Object(ObjectTypeDefinition::new(otd, cache))),
            TypeDefinitionReference::Interface(itd) => warden
                .is_interface_type_definition_visible(itd)
                .then(|| Self::Interface(InterfaceTypeDefinition::new(itd, cache))),
            TypeDefinitionReference::InputObject(iotd) => warden
                .is_input_object_type_definition_visible(iotd)
                .then(|| Self::InputObject(InputObjectTypeDefinition::new(iotd, cache))),
            TypeDefinitionReference::Enum(etd) => warden
                .is_enum_type_definition_visible(etd)
                .then(|| Self::Enum(EnumTypeDefinition::new(etd, cache))),
            TypeDefinitionReference::Union(utd) => warden
                .is_union_type_definition_visible(utd)
                .then(|| Self::Union(UnionTypeDefinition::new(utd, cache))),
        }
    }

    pub(crate) fn is_valid(&self) -> bool {
        match self {
            Self::BuiltinScalar(_) => true,
            Self::CustomScalar(_) => true,
            Self::Enum(etd) => !etd.enum_value_definitions().is_empty(),
            Self::InputObject(iotd) => !iotd.input_field_definitions().is_empty(),
            Self::Interface(itd) => !itd.fields_definition().is_empty(),
            Self::Object(otd) => !otd.fields_definition().is_empty(),
            Self::Union(utd) => !utd.union_member_types().is_empty(),
        }
    }
}

impl<'a, S: SchemaDefinitionWithVisibility + 'a> definition::TypeDefinition
    for TypeDefinition<'a, S>
{
    type ObjectTypeDefinition = ObjectTypeDefinition<'a, S>;
    type InputObjectTypeDefinition = InputObjectTypeDefinition<'a, S>;
    type CustomScalarTypeDefinition = ScalarTypeDefinition<'a, S>;
    type InterfaceTypeDefinition = InterfaceTypeDefinition<'a, S>;
    type EnumTypeDefinition = EnumTypeDefinition<'a, S>;
    type UnionTypeDefinition = UnionTypeDefinition<'a, S>;

    fn as_ref(&self) -> TypeDefinitionReference<'_, Self> {
        match self {
            Self::Object(otd) => TypeDefinitionReference::Object(otd),
            Self::Interface(itd) => TypeDefinitionReference::Interface(itd),
            Self::InputObject(iotd) => TypeDefinitionReference::InputObject(iotd),
            Self::CustomScalar(cstd) => TypeDefinitionReference::CustomScalar(cstd),
            Self::BuiltinScalar(bstd) => TypeDefinitionReference::BuiltinScalar(*bstd),
            Self::Enum(etd) => TypeDefinitionReference::Enum(etd),
            Self::Union(utd) => TypeDefinitionReference::Union(utd),
        }
    }
}
