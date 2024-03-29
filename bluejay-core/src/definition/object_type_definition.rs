use crate::definition::{FieldsDefinition, InterfaceImplementations};
use crate::ConstDirectives;

pub trait ObjectTypeDefinition {
    type FieldsDefinition: FieldsDefinition;
    type InterfaceImplementations: InterfaceImplementations;
    type Directives: ConstDirectives;

    fn description(&self) -> Option<&str>;
    fn name(&self) -> &str;
    fn interface_implementations(&self) -> Option<&Self::InterfaceImplementations>;
    fn directives(&self) -> Option<&Self::Directives>;
    fn fields_definition(&self) -> &Self::FieldsDefinition;
    fn is_builtin(&self) -> bool;
}
