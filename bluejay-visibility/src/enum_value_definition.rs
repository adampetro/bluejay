use crate::{Cache, Directives, SchemaDefinitionWithVisibility, Warden};
use bluejay_core::definition;

pub struct EnumValueDefinition<'a, S: SchemaDefinitionWithVisibility> {
    inner: &'a S::EnumValueDefinition,
    directives: Option<Directives<'a, S>>,
}

impl<'a, S: SchemaDefinitionWithVisibility> EnumValueDefinition<'a, S> {
    pub(crate) fn new(inner: &'a S::EnumValueDefinition, cache: &'a Cache<'a, S>) -> Option<Self> {
        cache
            .warden()
            .is_enum_value_definition_visible(inner)
            .then_some(Self {
                inner,
                directives: definition::EnumValueDefinition::directives(inner)
                    .map(|d| Directives::new(d, cache)),
            })
    }

    pub fn inner(&self) -> &'a S::EnumValueDefinition {
        self.inner
    }
}

impl<'a, S: SchemaDefinitionWithVisibility + 'a> definition::EnumValueDefinition
    for EnumValueDefinition<'a, S>
{
    type Directives = Directives<'a, S>;

    fn description(&self) -> Option<&str> {
        self.inner.description()
    }

    fn name(&self) -> &str {
        self.inner.name()
    }

    fn directives(&self) -> Option<&Self::Directives> {
        self.directives.as_ref()
    }
}
