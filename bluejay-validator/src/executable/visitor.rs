use bluejay_core::definition::{
    DirectiveLocation, SchemaDefinition, TypeDefinitionReferenceFromAbstract,
};
use bluejay_core::executable::{ExecutableDocument, OperationDefinitionFromExecutableDocument};

pub trait Visitor<'a, E: ExecutableDocument, S: SchemaDefinition> {
    fn visit_operation(
        &mut self,
        _operation_definition: &'a OperationDefinitionFromExecutableDocument<E>,
    ) {
    }

    fn visit_selection_set(
        &mut self,
        _selection_set: &'a E::SelectionSet,
        _type: &'a TypeDefinitionReferenceFromAbstract<S::TypeDefinitionReference>,
    ) {
    }

    fn visit_field(&mut self, _field: &'a E::Field, _field_definition: &'a S::FieldDefinition) {}

    fn visit_const_directive(
        &mut self,
        _directive: &'a E::Directive<true>,
        _location: DirectiveLocation,
    ) {
    }

    fn visit_variable_directive(
        &mut self,
        _directive: &'a E::Directive<false>,
        _location: DirectiveLocation,
    ) {
    }
}
