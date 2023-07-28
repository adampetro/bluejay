use crate::{Cache, InterfaceImplementation, SchemaDefinitionWithVisibility};
use bluejay_core::definition;
use bluejay_core::AsIter;
use once_cell::unsync::OnceCell;

pub struct InterfaceImplementations<'a, S: SchemaDefinitionWithVisibility> {
    inner: &'a S::InterfaceImplementations,
    cache: &'a Cache<'a, S>,
    interface_implementations: OnceCell<Vec<InterfaceImplementation<'a, S>>>,
}

impl<'a, S: SchemaDefinitionWithVisibility> InterfaceImplementations<'a, S> {
    pub(crate) fn new(inner: &'a S::InterfaceImplementations, cache: &'a Cache<'a, S>) -> Self {
        Self {
            inner,
            cache,
            interface_implementations: OnceCell::new(),
        }
    }
}

impl<'a, S: SchemaDefinitionWithVisibility + 'a> AsIter for InterfaceImplementations<'a, S> {
    type Item = InterfaceImplementation<'a, S>;
    type Iterator<'b> = std::slice::Iter<'b, Self::Item> where 'a: 'b;

    fn iter(&self) -> Self::Iterator<'_> {
        self.interface_implementations
            .get_or_init(|| {
                self.inner
                    .iter()
                    .filter_map(|ii| InterfaceImplementation::new(ii, self.cache))
                    .collect()
            })
            .iter()
    }
}

impl<'a, S: SchemaDefinitionWithVisibility + 'a> definition::InterfaceImplementations
    for InterfaceImplementations<'a, S>
{
    type InterfaceImplementation = InterfaceImplementation<'a, S>;
}
