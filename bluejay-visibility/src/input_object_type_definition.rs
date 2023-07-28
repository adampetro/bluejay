use crate::{Cache, Directives, InputFieldsDefinition, SchemaDefinitionWithVisibility};
use bluejay_core::definition;
use once_cell::unsync::OnceCell;

pub struct InputObjectTypeDefinition<'a, S: SchemaDefinitionWithVisibility> {
    inner: &'a S::InputObjectTypeDefinition,
    cache: &'a Cache<'a, S>,
    input_fields_definition: OnceCell<InputFieldsDefinition<'a, S>>,
    directives: Option<Directives<'a, S>>,
}

impl<'a, S: SchemaDefinitionWithVisibility + 'a> InputObjectTypeDefinition<'a, S> {
    pub fn new(inner: &'a S::InputObjectTypeDefinition, cache: &'a Cache<'a, S>) -> Self {
        Self {
            inner,
            cache,
            input_fields_definition: OnceCell::new(),
            directives: definition::InputObjectTypeDefinition::directives(inner)
                .map(|d| Directives::new(d, cache)),
        }
    }

    pub fn inner(&self) -> &'a S::InputObjectTypeDefinition {
        self.inner
    }

    pub fn cache(&self) -> &'a Cache<'a, S> {
        self.cache
    }
}

impl<'a, S: SchemaDefinitionWithVisibility + 'a> definition::InputObjectTypeDefinition
    for InputObjectTypeDefinition<'a, S>
{
    type Directives = Directives<'a, S>;
    type InputFieldsDefinition = InputFieldsDefinition<'a, S>;

    fn description(&self) -> Option<&str> {
        self.inner.description()
    }

    fn name(&self) -> &str {
        self.inner.name()
    }

    fn directives(&self) -> Option<&Self::Directives> {
        self.directives.as_ref()
    }

    fn input_field_definitions(&self) -> &Self::InputFieldsDefinition {
        self.input_fields_definition.get_or_init(|| {
            InputFieldsDefinition::new(self.inner.input_field_definitions(), self.cache)
        })
    }
}
