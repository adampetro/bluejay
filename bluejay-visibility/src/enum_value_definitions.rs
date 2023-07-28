use crate::{Cache, EnumValueDefinition, SchemaDefinitionWithVisibility};
use bluejay_core::definition;
use bluejay_core::AsIter;
use once_cell::unsync::OnceCell;

pub struct EnumValueDefinitions<'a, S: SchemaDefinitionWithVisibility> {
    inner: &'a S::EnumValueDefinitions,
    cache: &'a Cache<'a, S>,
    enum_value_definitions: OnceCell<Vec<EnumValueDefinition<'a, S>>>,
}

impl<'a, S: SchemaDefinitionWithVisibility> EnumValueDefinitions<'a, S> {
    pub(crate) fn new(inner: &'a S::EnumValueDefinitions, cache: &'a Cache<'a, S>) -> Self {
        Self {
            inner,
            cache,
            enum_value_definitions: OnceCell::new(),
        }
    }
}

impl<'a, S: SchemaDefinitionWithVisibility + 'a> AsIter for EnumValueDefinitions<'a, S> {
    type Item = EnumValueDefinition<'a, S>;
    type Iterator<'b> = std::slice::Iter<'b, Self::Item> where 'a: 'b;

    fn iter(&self) -> Self::Iterator<'_> {
        self.enum_value_definitions
            .get_or_init(|| {
                self.inner
                    .iter()
                    .filter_map(|evd| EnumValueDefinition::new(evd, self.cache))
                    .collect()
            })
            .iter()
    }
}

impl<'a, S: SchemaDefinitionWithVisibility + 'a> definition::EnumValueDefinitions
    for EnumValueDefinitions<'a, S>
{
    type EnumValueDefinition = EnumValueDefinition<'a, S>;
}
