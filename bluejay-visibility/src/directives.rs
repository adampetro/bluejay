use crate::{Cache, SchemaDefinitionWithVisibility};
use bluejay_core::{AsIter, Directive, Directives as CoreDirectives};
use once_cell::unsync::OnceCell;

pub struct Directives<'a, S: SchemaDefinitionWithVisibility> {
    inner: &'a S::Directives,
    cache: &'a Cache<'a, S>,
    directives: OnceCell<Vec<&'a <S::Directives as CoreDirectives<true>>::Directive>>,
}

impl<'a, S: SchemaDefinitionWithVisibility> Directives<'a, S> {
    pub(crate) fn new(inner: &'a S::Directives, cache: &'a Cache<'a, S>) -> Self {
        Self {
            inner,
            cache,
            directives: OnceCell::new(),
        }
    }
}

impl<'a, S: SchemaDefinitionWithVisibility + 'a> AsIter for Directives<'a, S> {
    type Item = <S::Directives as CoreDirectives<true>>::Directive;
    type Iterator<'b> = std::iter::Copied<std::slice::Iter<'b, &'b Self::Item>> where 'a: 'b;

    fn iter(&self) -> Self::Iterator<'_> {
        self.directives
            .get_or_init(|| {
                self.inner
                    .iter()
                    .filter_map(|directive| {
                        self.cache
                            .get_directive_definition(directive.name())
                            .map(|_| directive)
                    })
                    .collect()
            })
            .iter()
            .copied()
    }
}

impl<'a, S: SchemaDefinitionWithVisibility + 'a> CoreDirectives<true> for Directives<'a, S> {
    type Directive = <S::Directives as CoreDirectives<true>>::Directive;
}
