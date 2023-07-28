use crate::{Cache, Directives, EnumValueDefinitions, SchemaDefinitionWithVisibility};
use bluejay_core::definition;
use once_cell::unsync::OnceCell;

pub struct EnumTypeDefinition<'a, S: SchemaDefinitionWithVisibility> {
    inner: &'a S::EnumTypeDefinition,
    cache: &'a Cache<'a, S>,
    enum_value_definitions: OnceCell<EnumValueDefinitions<'a, S>>,
    directives: Option<Directives<'a, S>>,
}

impl<'a, S: SchemaDefinitionWithVisibility + 'a> EnumTypeDefinition<'a, S> {
    pub(crate) fn new(inner: &'a S::EnumTypeDefinition, cache: &'a Cache<'a, S>) -> Self {
        Self {
            inner,
            cache,
            enum_value_definitions: OnceCell::new(),
            directives: definition::EnumTypeDefinition::directives(inner)
                .map(|d| Directives::new(d, cache)),
        }
    }
}

impl<'a, S: SchemaDefinitionWithVisibility + 'a> definition::EnumTypeDefinition
    for EnumTypeDefinition<'a, S>
{
    type Directives = Directives<'a, S>;
    type EnumValueDefinitions = EnumValueDefinitions<'a, S>;

    fn description(&self) -> Option<&str> {
        self.inner.description()
    }

    fn name(&self) -> &str {
        self.inner.name()
    }

    fn directives(&self) -> Option<&Self::Directives> {
        self.directives.as_ref()
    }

    fn enum_value_definitions(&self) -> &Self::EnumValueDefinitions {
        self.enum_value_definitions.get_or_init(|| {
            EnumValueDefinitions::new(self.inner.enum_value_definitions(), self.cache)
        })
    }

    fn is_builtin(&self) -> bool {
        self.inner.is_builtin()
    }
}
