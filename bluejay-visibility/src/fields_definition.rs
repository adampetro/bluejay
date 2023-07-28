use crate::{Cache, FieldDefinition, SchemaDefinitionWithVisibility};
use bluejay_core::definition;
use bluejay_core::AsIter;
use once_cell::unsync::OnceCell;

pub struct FieldsDefinition<'a, S: SchemaDefinitionWithVisibility> {
    inner: &'a S::FieldsDefinition,
    cache: &'a Cache<'a, S>,
    fields_definition: OnceCell<Vec<FieldDefinition<'a, S>>>,
}

impl<'a, S: SchemaDefinitionWithVisibility> FieldsDefinition<'a, S> {
    pub(crate) fn new(inner: &'a S::FieldsDefinition, cache: &'a Cache<'a, S>) -> Self {
        Self {
            inner,
            cache,
            fields_definition: OnceCell::new(),
        }
    }
}

impl<'a, S: SchemaDefinitionWithVisibility + 'a> AsIter for FieldsDefinition<'a, S> {
    type Item = FieldDefinition<'a, S>;
    type Iterator<'b> = std::slice::Iter<'b, Self::Item> where 'a: 'b;

    fn iter(&self) -> Self::Iterator<'_> {
        self.fields_definition
            .get_or_init(|| {
                self.inner
                    .iter()
                    .filter_map(|fd| FieldDefinition::new(fd, self.cache))
                    .collect()
            })
            .iter()
    }
}

impl<'a, S: SchemaDefinitionWithVisibility + 'a> definition::FieldsDefinition
    for FieldsDefinition<'a, S>
{
    type FieldDefinition = FieldDefinition<'a, S>;
}
