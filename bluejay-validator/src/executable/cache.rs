use crate::executable::VariableDefinitionInputType;
use bluejay_core::definition::SchemaDefinition;
use bluejay_core::executable::{
    ExecutableDocument, FragmentDefinition, OperationDefinition, VariableDefinition,
};
use bluejay_core::AsIter;
use std::collections::HashMap;

pub struct Cache<'a, E: ExecutableDocument, S: SchemaDefinition> {
    variable_definition_input_types:
        HashMap<&'a E::VariableType, VariableDefinitionInputType<'a, S::BaseInputType>>,
    indexed_fragment_definitions: HashMap<&'a str, &'a E::FragmentDefinition>,
}

impl<'a, E: ExecutableDocument, S: SchemaDefinition> Cache<'a, E, S> {
    pub fn new(executable_document: &'a E, schema_definition: &'a S) -> Self {
        let variable_definition_input_types =
            HashMap::from_iter(executable_document.operation_definitions().iter().flat_map(
                |operation_definition: &'a E::OperationDefinition| {
                    let variable_definitions_iterator = operation_definition
                        .as_ref()
                        .variable_definitions()
                        .map(|variable_definitions: &'a E::VariableDefinitions| -> <E::VariableDefinitions as AsIter>::Iterator<'a> {
                            variable_definitions.iter()
                        });

                    variable_definitions_iterator
                        .into_iter()
                        .flatten()
                        .filter_map(|variable_definition| {
                            let variable_type = variable_definition.r#type();
                            VariableDefinitionInputType::try_from((
                                schema_definition,
                                variable_type,
                            ))
                            .ok()
                            .map(|vdit| (variable_type, vdit))
                        })
                },
            ));
        let indexed_fragment_definitions = HashMap::from_iter(
            executable_document
                .fragment_definitions()
                .as_ref()
                .iter()
                .map(|fragment_definition| (fragment_definition.name(), fragment_definition)),
        );
        Self {
            variable_definition_input_types,
            indexed_fragment_definitions,
        }
    }

    pub fn variable_definition_input_type(
        &self,
        variable_type: &E::VariableType,
    ) -> Option<&VariableDefinitionInputType<'a, S::BaseInputType>> {
        self.variable_definition_input_types.get(variable_type)
    }

    pub fn fragment_definition(&self, name: &str) -> Option<&'a E::FragmentDefinition> {
        self.indexed_fragment_definitions.get(name).copied()
    }
}
