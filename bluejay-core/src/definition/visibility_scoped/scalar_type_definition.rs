use crate::definition::{self, visibility_scoped::Warden, SchemaDefinition};
use std::marker::PhantomData;

pub struct ScalarTypeDefinition<'a, S: SchemaDefinition, W: Warden<SchemaDefinition = S>> {
    inner: &'a S::CustomScalarTypeDefinition,
    warden: PhantomData<W>,
}

impl<'a, S: SchemaDefinition + 'a, W: Warden<SchemaDefinition = S>> ScalarTypeDefinition<'a, S, W> {
    pub(crate) fn new(inner: &'a S::CustomScalarTypeDefinition) -> Self {
        Self {
            inner,
            warden: Default::default(),
        }
    }
}

impl<'a, S: SchemaDefinition + 'a, W: Warden<SchemaDefinition = S>> definition::ScalarTypeDefinition
    for ScalarTypeDefinition<'a, S, W>
{
    type Directives =
        <S::CustomScalarTypeDefinition as definition::ScalarTypeDefinition>::Directives;

    fn description(&self) -> Option<&str> {
        self.inner.description()
    }

    fn name(&self) -> &str {
        self.inner.name()
    }

    fn directives(&self) -> Option<&Self::Directives> {
        self.inner.directives()
    }

    fn coerce_input<const CONST: bool>(
        &self,
        value: &impl crate::Value<CONST>,
    ) -> Result<(), std::borrow::Cow<'static, str>> {
        self.inner.coerce_input(value)
    }
}
