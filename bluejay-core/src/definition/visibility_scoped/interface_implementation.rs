use crate::definition::{
    self,
    visibility_scoped::{Cache, InterfaceTypeDefinition, Warden},
    SchemaDefinition, TypeDefinitionReference,
};
use once_cell::unsync::OnceCell;

pub struct InterfaceImplementation<'a, S: SchemaDefinition, W: Warden<SchemaDefinition = S>> {
    inner: &'a S::InterfaceImplementation,
    cache: &'a Cache<'a, S, W>,
    interface: OnceCell<&'a InterfaceTypeDefinition<'a, S, W>>,
}

impl<'a, S: SchemaDefinition, W: Warden<SchemaDefinition = S>> InterfaceImplementation<'a, S, W> {
    pub(crate) fn new(
        inner: &'a S::InterfaceImplementation,
        cache: &'a Cache<'a, S, W>,
    ) -> Option<Self> {
        cache
            .warden()
            .is_interface_implementation_visible(inner)
            .then_some(Self {
                inner,
                cache,
                interface: OnceCell::new(),
            })
    }
}

impl<'a, S: SchemaDefinition + 'a, W: Warden<SchemaDefinition = S>>
    definition::InterfaceImplementation for InterfaceImplementation<'a, S, W>
{
    type InterfaceTypeDefinition = InterfaceTypeDefinition<'a, S, W>;

    fn interface(&self) -> &Self::InterfaceTypeDefinition {
        self.interface.get_or_init(|| {
            self.cache
                .get_or_create_type_definition(TypeDefinitionReference::Interface(
                    self.inner.interface(),
                ))
                .as_interface()
                .unwrap()
        })
    }
}
