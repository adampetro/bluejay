use crate::{Cache, SchemaDefinitionWithVisibility, UnionMemberType};
use bluejay_core::definition;
use bluejay_core::AsIter;
use once_cell::unsync::OnceCell;

pub struct UnionMemberTypes<'a, S: SchemaDefinitionWithVisibility> {
    inner: &'a S::UnionMemberTypes,
    cache: &'a Cache<'a, S>,
    member_types: OnceCell<Vec<UnionMemberType<'a, S>>>,
}

impl<'a, S: SchemaDefinitionWithVisibility> UnionMemberTypes<'a, S> {
    pub(crate) fn new(inner: &'a S::UnionMemberTypes, cache: &'a Cache<'a, S>) -> Self {
        Self {
            inner,
            cache,
            member_types: OnceCell::new(),
        }
    }
}

impl<'a, S: SchemaDefinitionWithVisibility + 'a> AsIter for UnionMemberTypes<'a, S> {
    type Item = UnionMemberType<'a, S>;
    type Iterator<'b> = std::slice::Iter<'b, Self::Item> where 'a: 'b;

    fn iter(&self) -> Self::Iterator<'_> {
        self.member_types
            .get_or_init(|| {
                self.inner
                    .iter()
                    .filter_map(|mt| UnionMemberType::new(mt, self.cache))
                    .collect()
            })
            .iter()
    }
}

impl<'a, S: SchemaDefinitionWithVisibility + 'a> definition::UnionMemberTypes
    for UnionMemberTypes<'a, S>
{
    type UnionMemberType = UnionMemberType<'a, S>;
}
