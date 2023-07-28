use crate::{Cache, InputValueDefinition, SchemaDefinitionWithVisibility};
use bluejay_core::definition;
use bluejay_core::AsIter;
use once_cell::unsync::OnceCell;

pub struct ArgumentsDefinition<'a, S: SchemaDefinitionWithVisibility> {
    inner: &'a S::ArgumentsDefinition,
    cache: &'a Cache<'a, S>,
    arguments_definition: OnceCell<Vec<InputValueDefinition<'a, S>>>,
}

impl<'a, S: SchemaDefinitionWithVisibility> ArgumentsDefinition<'a, S> {
    pub(crate) fn new(inner: &'a S::ArgumentsDefinition, cache: &'a Cache<'a, S>) -> Self {
        Self {
            inner,
            cache,
            arguments_definition: OnceCell::new(),
        }
    }
}

impl<'a, S: SchemaDefinitionWithVisibility + 'a> AsIter for ArgumentsDefinition<'a, S> {
    type Item = InputValueDefinition<'a, S>;
    type Iterator<'b> = std::slice::Iter<'b, Self::Item> where 'a: 'b;

    fn iter(&self) -> Self::Iterator<'_> {
        self.arguments_definition
            .get_or_init(|| {
                self.inner
                    .iter()
                    .filter_map(|ivd| InputValueDefinition::new(ivd, self.cache))
                    .collect()
            })
            .iter()
    }
}

impl<'a, S: SchemaDefinitionWithVisibility + 'a> definition::ArgumentsDefinition
    for ArgumentsDefinition<'a, S>
{
    type ArgumentDefinition = InputValueDefinition<'a, S>;
}
