use crate::{Cache, ObjectTypeDefinition, SchemaDefinitionWithVisibility, Warden};
use bluejay_core::definition::{self, TypeDefinitionReference};

pub struct UnionMemberType<'a, S: SchemaDefinitionWithVisibility> {
    inner: &'a S::UnionMemberType,
    member_type: &'a ObjectTypeDefinition<'a, S>,
}

impl<'a, S: SchemaDefinitionWithVisibility> UnionMemberType<'a, S> {
    pub(crate) fn new(inner: &'a S::UnionMemberType, cache: &'a Cache<'a, S>) -> Option<Self> {
        cache
            .warden()
            .is_union_member_type_visible(inner)
            .then(|| {
                cache
                    .get_or_create_type_definition(TypeDefinitionReference::Object(
                        definition::UnionMemberType::member_type(inner),
                    ))
                    .map(|td| Self {
                        inner,
                        member_type: td.as_object().unwrap(),
                    })
            })
            .flatten()
    }

    pub fn inner(&self) -> &'a S::UnionMemberType {
        self.inner
    }
}

impl<'a, S: SchemaDefinitionWithVisibility + 'a> definition::UnionMemberType
    for UnionMemberType<'a, S>
{
    type ObjectTypeDefinition = ObjectTypeDefinition<'a, S>;

    fn member_type(&self) -> &Self::ObjectTypeDefinition {
        self.member_type
    }
}
