use crate::{Cache, InputValueDefinition, SchemaDefinitionWithVisibility};
use bluejay_core::definition;
use bluejay_core::AsIter;
use once_cell::unsync::OnceCell;

pub struct InputFieldsDefinition<'a, S: SchemaDefinitionWithVisibility> {
    inner: &'a S::InputFieldsDefinition,
    cache: &'a Cache<'a, S>,
    input_fields_definition: OnceCell<Vec<InputValueDefinition<'a, S>>>,
}

impl<'a, S: SchemaDefinitionWithVisibility> InputFieldsDefinition<'a, S> {
    pub(crate) fn new(inner: &'a S::InputFieldsDefinition, cache: &'a Cache<'a, S>) -> Self {
        Self {
            inner,
            cache,
            input_fields_definition: OnceCell::new(),
        }
    }
}

impl<'a, S: SchemaDefinitionWithVisibility + 'a> AsIter for InputFieldsDefinition<'a, S> {
    type Item = InputValueDefinition<'a, S>;
    type Iterator<'b> = std::slice::Iter<'b, Self::Item> where 'a: 'b;

    fn iter(&self) -> Self::Iterator<'_> {
        self.input_fields_definition
            .get_or_init(|| {
                self.inner
                    .iter()
                    .filter_map(|ivd| InputValueDefinition::new(ivd, self.cache))
                    .collect()
            })
            .iter()
    }
}

impl<'a, S: SchemaDefinitionWithVisibility + 'a> definition::InputFieldsDefinition
    for InputFieldsDefinition<'a, S>
{
    type InputValueDefinition = InputValueDefinition<'a, S>;
}
