use bluejay_core::definition::{InputObjectTypeDefinition, SchemaDefinition};

#[cfg(feature = "parser-integration")]
use bluejay_parser::{
    ast::definition::SchemaDefinition as ParserSchemaDefinition,
    error::{Annotation, Error as ParserError},
    HasSpan,
};

pub enum Error<'a, S: SchemaDefinition> {
    NonUniqueInputValueDefinitionNames {
        name: &'a str,
        input_value_definitions: Vec<&'a S::InputValueDefinition>,
    },
    NonUniqueEnumValueDefinitionNames {
        name: &'a str,
        enum_value_definitions: Vec<&'a S::EnumValueDefinition>,
    },
    InputObjectTypeDefinitionCircularReferences {
        input_object_type_definition: &'a S::InputObjectTypeDefinition,
        circular_references: Vec<&'a S::InputType>,
    },
}

#[cfg(feature = "parser-integration")]
impl<'a> From<Error<'a, ParserSchemaDefinition<'a>>> for ParserError {
    fn from(value: Error<'a, ParserSchemaDefinition<'a>>) -> Self {
        match value {
            Error::NonUniqueInputValueDefinitionNames {
                name,
                input_value_definitions,
            } => Self::new(
                format!("Multiple input value definitions named `{name}`"),
                None,
                input_value_definitions
                    .into_iter()
                    .map(|ivd| {
                        Annotation::new(
                            format!("Input value definition with name `{name}`"),
                            ivd.name_token().span().clone(),
                        )
                    })
                    .collect(),
            ),
            Error::NonUniqueEnumValueDefinitionNames {
                name,
                enum_value_definitions,
            } => Self::new(
                format!("Multiple enum value definitions named `{name}`"),
                None,
                enum_value_definitions
                    .into_iter()
                    .map(|evd| {
                        Annotation::new(
                            format!("Enum value definition with name `{name}`"),
                            evd.name_token().span().clone(),
                        )
                    })
                    .collect(),
            ),
            Error::InputObjectTypeDefinitionCircularReferences {
                input_object_type_definition,
                circular_references,
            } => {
                Self::new(
                format!(
                    "Input object type definition `{}` contains disallowed circular reference(s)",
                    input_object_type_definition.name()
                ),
                Some(Annotation::new(
                    "Input object type definition contains circular reference(s) through an unbroken chain of non-null singular fields, which is disallowed",
                    input_object_type_definition.name_token().span().clone(),
                )),
                circular_references
                    .into_iter()
                    .map(|circular_reference| {
                        Annotation::new(
                            "Occurence of circular reference",
                            circular_reference.span().clone(),
                        )
                    })
                    .collect(),
            )
            }
        }
    }
}
