use crate::{Cache, InterfaceTypeDefinition, SchemaDefinitionWithVisibility, Warden};
use bluejay_core::definition::{self, TypeDefinitionReference};

pub struct InterfaceImplementation<'a, S: SchemaDefinitionWithVisibility> {
    inner: &'a S::InterfaceImplementation,
    interface: &'a InterfaceTypeDefinition<'a, S>,
}

impl<'a, S: SchemaDefinitionWithVisibility> InterfaceImplementation<'a, S> {
    pub(crate) fn new(
        inner: &'a S::InterfaceImplementation,
        cache: &'a Cache<'a, S>,
    ) -> Option<Self> {
        cache
            .warden()
            .is_interface_implementation_visible(inner)
            .then(|| {
                cache
                    .get_or_create_type_definition(TypeDefinitionReference::Interface(
                        definition::InterfaceImplementation::interface(inner),
                    ))
                    .map(|td| Self {
                        inner,
                        interface: td.as_interface().unwrap(),
                    })
            })
            .flatten()
    }

    pub fn inner(&self) -> &'a S::InterfaceImplementation {
        self.inner
    }
}

impl<'a, S: SchemaDefinitionWithVisibility + 'a> definition::InterfaceImplementation
    for InterfaceImplementation<'a, S>
{
    type InterfaceTypeDefinition = InterfaceTypeDefinition<'a, S>;

    fn interface(&self) -> &Self::InterfaceTypeDefinition {
        self.interface
    }
}
