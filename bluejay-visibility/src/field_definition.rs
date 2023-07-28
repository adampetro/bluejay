use crate::{
    ArgumentsDefinition, Cache, Directives, OutputType, SchemaDefinitionWithVisibility, Warden,
};
use bluejay_core::definition;
use once_cell::unsync::OnceCell;

pub struct FieldDefinition<'a, S: SchemaDefinitionWithVisibility> {
    inner: &'a S::FieldDefinition,
    cache: &'a Cache<'a, S>,
    r#type: OutputType<'a, S>,
    arguments_definition: OnceCell<Option<ArgumentsDefinition<'a, S>>>,
    directives: Option<Directives<'a, S>>,
}

impl<'a, S: SchemaDefinitionWithVisibility> FieldDefinition<'a, S> {
    pub(crate) fn new(inner: &'a S::FieldDefinition, cache: &'a Cache<'a, S>) -> Option<Self> {
        cache
            .warden()
            .is_field_definition_visible(inner)
            .then(|| {
                OutputType::new(definition::FieldDefinition::r#type(inner), cache).map(|r#type| {
                    Self {
                        inner,
                        cache,
                        r#type,
                        arguments_definition: OnceCell::new(),
                        directives: definition::FieldDefinition::directives(inner)
                            .map(|d| Directives::new(d, cache)),
                    }
                })
            })
            .flatten()
    }

    pub fn inner(&self) -> &'a S::FieldDefinition {
        self.inner
    }
}

impl<'a, S: SchemaDefinitionWithVisibility + 'a> definition::FieldDefinition
    for FieldDefinition<'a, S>
{
    type OutputType = OutputType<'a, S>;
    type Directives = Directives<'a, S>;
    type ArgumentsDefinition = ArgumentsDefinition<'a, S>;

    fn description(&self) -> Option<&str> {
        self.inner.description()
    }

    fn name(&self) -> &str {
        self.inner.name()
    }

    fn is_builtin(&self) -> bool {
        self.inner.is_builtin()
    }

    fn r#type(&self) -> &Self::OutputType {
        &self.r#type
    }

    fn directives(&self) -> Option<&Self::Directives> {
        self.directives.as_ref()
    }

    fn arguments_definition(&self) -> Option<&Self::ArgumentsDefinition> {
        self.arguments_definition
            .get_or_init(|| {
                self.inner
                    .arguments_definition()
                    .map(|ad| ArgumentsDefinition::new(ad, self.cache))
            })
            .as_ref()
    }
}
