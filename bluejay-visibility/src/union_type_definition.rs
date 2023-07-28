use crate::{
    Cache, Directives, FieldsDefinition, SchemaDefinitionWithVisibility, UnionMemberTypes,
};
use bluejay_core::definition;
use once_cell::unsync::OnceCell;

pub struct UnionTypeDefinition<'a, S: SchemaDefinitionWithVisibility> {
    inner: &'a S::UnionTypeDefinition,
    cache: &'a Cache<'a, S>,
    union_member_types: OnceCell<UnionMemberTypes<'a, S>>,
    fields_definition: OnceCell<FieldsDefinition<'a, S>>,
    directives: Option<Directives<'a, S>>,
}

impl<'a, S: SchemaDefinitionWithVisibility + 'a> UnionTypeDefinition<'a, S> {
    pub(crate) fn new(inner: &'a S::UnionTypeDefinition, cache: &'a Cache<'a, S>) -> Self {
        Self {
            inner,
            cache,
            union_member_types: OnceCell::new(),
            fields_definition: OnceCell::new(),
            directives: definition::UnionTypeDefinition::directives(inner)
                .map(|d| Directives::new(d, cache)),
        }
    }

    pub fn inner(&self) -> &'a S::UnionTypeDefinition {
        self.inner
    }
}

impl<'a, S: SchemaDefinitionWithVisibility + 'a> definition::UnionTypeDefinition
    for UnionTypeDefinition<'a, S>
{
    type Directives = Directives<'a, S>;
    type UnionMemberTypes = UnionMemberTypes<'a, S>;
    type FieldsDefinition = FieldsDefinition<'a, S>;

    fn description(&self) -> Option<&str> {
        self.inner.description()
    }

    fn name(&self) -> &str {
        self.inner.name()
    }

    fn directives(&self) -> Option<&Self::Directives> {
        self.directives.as_ref()
    }

    fn union_member_types(&self) -> &Self::UnionMemberTypes {
        self.union_member_types
            .get_or_init(|| UnionMemberTypes::new(self.inner.union_member_types(), self.cache))
    }

    fn fields_definition(&self) -> &Self::FieldsDefinition {
        self.fields_definition
            .get_or_init(|| FieldsDefinition::new(self.inner.fields_definition(), self.cache))
    }
}
