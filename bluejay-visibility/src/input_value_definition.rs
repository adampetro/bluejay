use crate::{Cache, Directives, InputType, SchemaDefinitionWithVisibility, Warden};
use bluejay_core::definition::{self, InputValueDefinition as _, SchemaDefinition};

pub trait InputValueDefinitionWithVisibility: definition::InputValueDefinition {
    type SchemaDefinition: SchemaDefinitionWithVisibility;

    #[allow(unused_variables)]
    fn default_value<'a>(
        &'a self,
        warden: &'a Cache<'a, Self::SchemaDefinition>,
    ) -> Option<&Self::Value> {
        definition::InputValueDefinition::default_value(self)
    }
}

pub struct InputValueDefinition<'a, S: SchemaDefinitionWithVisibility> {
    inner: &'a <S as SchemaDefinitionWithVisibility>::InputValueDefinition,
    r#type: InputType<'a, S>,
    directives: Option<Directives<'a, S>>,
    cache: &'a Cache<'a, S>,
}

impl<'a, S: SchemaDefinitionWithVisibility> InputValueDefinition<'a, S> {
    pub(crate) fn new(
        inner: &'a <S as SchemaDefinitionWithVisibility>::InputValueDefinition,
        cache: &'a Cache<'a, S>,
    ) -> Option<Self> {
        cache
            .warden()
            .is_input_value_definition_visible(inner)
            .then(|| {
                (InputType::new(inner.r#type(), cache) as Option<InputType<'a, S>>).map(|r#type| {
                    Self {
                        inner,
                        r#type,
                        directives: definition::InputValueDefinition::directives(inner)
                            .map(|d| Directives::new(d, cache)),
                        cache,
                    }
                })
            })
            .flatten()
    }

    pub fn inner(&self) -> &'a <S as SchemaDefinition>::InputValueDefinition {
        self.inner
    }
}

impl<'a, S: SchemaDefinitionWithVisibility + 'a> definition::InputValueDefinition
    for InputValueDefinition<'a, S>
{
    type Value =
        <<S as SchemaDefinitionWithVisibility>::InputValueDefinition as definition::InputValueDefinition>::Value;
    type Directives = Directives<'a, S>;
    type InputType = InputType<'a, S>;

    fn description(&self) -> Option<&str> {
        self.inner.description()
    }

    fn name(&self) -> &str {
        self.inner.name()
    }

    fn default_value(&self) -> Option<&Self::Value> {
        InputValueDefinitionWithVisibility::default_value(self.inner, self.cache)
    }

    fn directives(&self) -> Option<&Self::Directives> {
        self.directives.as_ref()
    }

    fn r#type(&self) -> &Self::InputType {
        &self.r#type
    }
}
