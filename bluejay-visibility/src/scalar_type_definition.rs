use crate::{Cache, Directives, SchemaDefinitionWithVisibility};
use bluejay_core::definition;

pub struct ScalarTypeDefinition<'a, S: SchemaDefinitionWithVisibility> {
    inner: &'a S::CustomScalarTypeDefinition,
    directives: Option<Directives<'a, S>>,
}

impl<'a, S: SchemaDefinitionWithVisibility + 'a> ScalarTypeDefinition<'a, S> {
    pub(crate) fn new(inner: &'a S::CustomScalarTypeDefinition, cache: &'a Cache<'a, S>) -> Self {
        Self {
            inner,
            directives: definition::ScalarTypeDefinition::directives(inner)
                .map(|d| Directives::new(d, cache)),
        }
    }

    pub fn inner(&self) -> &'a S::CustomScalarTypeDefinition {
        self.inner
    }
}

impl<'a, S: SchemaDefinitionWithVisibility + 'a> definition::ScalarTypeDefinition
    for ScalarTypeDefinition<'a, S>
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

    fn coerce_input<const CONST: bool>(
        &self,
        value: &impl bluejay_core::Value<CONST>,
    ) -> Result<(), std::borrow::Cow<'static, str>> {
        self.inner.coerce_input(value)
    }
}
