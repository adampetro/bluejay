use crate::definition::{
    prelude::*,
    visibility_scoped::{DirectiveDefinition, TypeDefinition, Warden},
    SchemaDefinition, TypeDefinitionReference,
};
use elsa::FrozenMap;

pub struct Cache<'a, S: SchemaDefinition, W: Warden<SchemaDefinition = S>> {
    schema_definition: &'a S,
    warden: &'a W,
    type_definitions: FrozenMap<&'a str, Box<TypeDefinition<'a, S, W>>>,
    directive_definitions: FrozenMap<&'a str, Box<DirectiveDefinition<'a, S, W>>>,
}

impl<'a, S: SchemaDefinition, W: Warden<SchemaDefinition = S>> Cache<'a, S, W> {
    pub fn new(schema_definition: &'a S, warden: &'a W) -> Self {
        Self {
            schema_definition,
            warden,
            type_definitions: FrozenMap::new(),
            directive_definitions: FrozenMap::new(),
        }
    }

    pub(crate) fn schema_definition(&self) -> &'a S {
        self.schema_definition
    }

    pub(crate) fn warden(&self) -> &'a W {
        self.warden
    }

    pub(crate) fn get_or_create_type_definition(
        &'a self,
        type_definition: impl Into<TypeDefinitionReference<'a, S::TypeDefinition>>,
    ) -> &'a TypeDefinition<'a, S, W> {
        let type_definition = type_definition.into();
        self.type_definitions
            .get(type_definition.name())
            .unwrap_or_else(|| {
                self.type_definitions.insert(
                    type_definition.name(),
                    Box::new(TypeDefinition::new(type_definition, self)),
                )
            })
    }

    pub(crate) fn get_or_create_type_definition_by_name(
        &'a self,
        name: &str,
    ) -> Option<&'a TypeDefinition<'a, S, W>> {
        self.type_definitions.get(name).or_else(|| {
            self.schema_definition.get_type_definition(name).map(|td| {
                self.type_definitions
                    .insert(td.name(), Box::new(TypeDefinition::new(td, self)))
            })
        })
    }

    pub(crate) fn get_or_create_directive_definition(
        &'a self,
        directive_definition: &'a S::DirectiveDefinition,
    ) -> &'a DirectiveDefinition<'a, S, W> {
        self.directive_definitions
            .get(directive_definition.name())
            .unwrap_or_else(|| {
                self.directive_definitions.insert(
                    directive_definition.name(),
                    Box::new(DirectiveDefinition::new(directive_definition, self)),
                )
            })
    }

    pub(crate) fn get_or_create_directive_definition_by_name(
        &'a self,
        name: &str,
    ) -> Option<&'a DirectiveDefinition<'a, S, W>> {
        self.directive_definitions.get(name).or_else(|| {
            self.schema_definition
                .get_directive_definition(name)
                .map(|dd| {
                    self.directive_definitions
                        .insert(dd.name(), Box::new(DirectiveDefinition::new(dd, self)))
                })
        })
    }
}
