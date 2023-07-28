use crate::{ArgumentsDefinition, Cache, SchemaDefinitionWithVisibility};
use bluejay_core::definition;
use once_cell::unsync::OnceCell;

pub struct DirectiveDefinition<'a, S: SchemaDefinitionWithVisibility> {
    inner: &'a S::DirectiveDefinition,
    cache: &'a Cache<'a, S>,
    arguments_definition: OnceCell<Option<ArgumentsDefinition<'a, S>>>,
}

impl<'a, S: SchemaDefinitionWithVisibility + 'a> DirectiveDefinition<'a, S> {
    pub fn new(inner: &'a S::DirectiveDefinition, cache: &'a Cache<'a, S>) -> Self {
        Self {
            inner,
            cache,
            arguments_definition: OnceCell::new(),
        }
    }

    pub fn inner(&self) -> &'a S::DirectiveDefinition {
        self.inner
    }
}

impl<'a, S: SchemaDefinitionWithVisibility + 'a> definition::DirectiveDefinition
    for DirectiveDefinition<'a, S>
{
    type ArgumentsDefinition = ArgumentsDefinition<'a, S>;
    type DirectiveLocations =
        <S::DirectiveDefinition as definition::DirectiveDefinition>::DirectiveLocations;

    fn description(&self) -> Option<&str> {
        self.inner.description()
    }

    fn name(&self) -> &str {
        self.inner.name()
    }

    fn is_builtin(&self) -> bool {
        self.inner.is_builtin()
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

    fn is_repeatable(&self) -> bool {
        self.inner.is_repeatable()
    }

    fn locations(&self) -> &Self::DirectiveLocations {
        self.inner.locations()
    }
}
