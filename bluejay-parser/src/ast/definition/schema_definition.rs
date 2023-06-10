use crate::ast::definition::{
    ArgumentsDefinition, BaseInputType, BaseOutputType, Context, CustomScalarTypeDefinition,
    DefaultContext, DirectiveDefinition, EnumTypeDefinition, EnumValueDefinition,
    EnumValueDefinitions, FieldDefinition, FieldsDefinition, InputFieldsDefinition,
    InputObjectTypeDefinition, InputType, InputValueDefinition, InterfaceImplementation,
    InterfaceImplementations, InterfaceTypeDefinition, ObjectTypeDefinition, OutputType,
    TypeDefinition, UnionMemberType, UnionMemberTypes, UnionTypeDefinition,
};
use crate::ast::ConstDirectives;
use crate::lexical_token::StringValue;
use bluejay_core::definition::{
    InterfaceImplementation as CoreInterfaceImplementation,
    ObjectTypeDefinition as CoreObjectTypeDefinition, SchemaDefinition as CoreSchemaDefinition,
    TypeDefinition as CoreTypeDefinition, TypeDefinitionReference,
};
use bluejay_core::AsIter;
use std::collections::{btree_map::Values, BTreeMap, HashMap};
use std::sync::Arc;

#[derive(Debug)]
pub struct SchemaDefinition<'a, C: Context = DefaultContext> {
    type_definitions: BTreeMap<&'a str, TypeDefinition<'a, C>>,
    directive_definitions: BTreeMap<&'a str, DirectiveDefinition<'a, C>>,
    description: Option<StringValue<'a>>,
    query: Arc<ObjectTypeDefinition<'a, C>>,
    mutation: Option<Arc<ObjectTypeDefinition<'a, C>>>,
    subscription: Option<Arc<ObjectTypeDefinition<'a, C>>>,
    schema_directives: Option<ConstDirectives<'a>>,
    interface_implementors: HashMap<&'a str, Vec<Arc<ObjectTypeDefinition<'a, C>>>>,
}

impl<'a, C: Context> SchemaDefinition<'a, C> {
    pub(crate) fn new(
        type_definitions: BTreeMap<&'a str, TypeDefinition<'a, C>>,
        directive_definitions: BTreeMap<&'a str, DirectiveDefinition<'a, C>>,
        description: Option<StringValue<'a>>,
        query: Arc<ObjectTypeDefinition<'a, C>>,
        mutation: Option<Arc<ObjectTypeDefinition<'a, C>>>,
        subscription: Option<Arc<ObjectTypeDefinition<'a, C>>>,
        schema_directives: Option<ConstDirectives<'a>>,
    ) -> Self {
        let interface_implementors = Self::interface_implementors(&type_definitions);
        Self {
            type_definitions,
            directive_definitions,
            description,
            query,
            mutation,
            subscription,
            schema_directives,
            interface_implementors,
        }
    }

    fn interface_implementors(
        type_definitions: &BTreeMap<&'a str, TypeDefinition<'a, C>>,
    ) -> HashMap<&'a str, Vec<Arc<ObjectTypeDefinition<'a, C>>>> {
        type_definitions.values().fold(
            HashMap::new(),
            |mut interface_implementors, type_definition| {
                if let TypeDefinition::Object(otd) = type_definition {
                    if let Some(interface_implementations) = otd.interface_implementations() {
                        interface_implementations
                            .iter()
                            .for_each(|interface_implementation| {
                                let itd = interface_implementation.interface();
                                interface_implementors
                                    .entry(itd.name().as_str())
                                    .or_default()
                                    .push(otd.clone());
                            });
                    }
                }

                interface_implementors
            },
        )
    }
}

impl<'a, C: Context> CoreSchemaDefinition for SchemaDefinition<'a, C> {
    type Directives = ConstDirectives<'a>;
    type InputValueDefinition = InputValueDefinition<'a, C>;
    type InputFieldsDefinition = InputFieldsDefinition<'a, C>;
    type ArgumentsDefinition = ArgumentsDefinition<'a, C>;
    type EnumValueDefinition = EnumValueDefinition<'a>;
    type EnumValueDefinitions = EnumValueDefinitions<'a>;
    type FieldDefinition = FieldDefinition<'a, C>;
    type FieldsDefinition = FieldsDefinition<'a, C>;
    type InterfaceImplementation = InterfaceImplementation<'a, C>;
    type InterfaceImplementations = InterfaceImplementations<'a, C>;
    type UnionMemberType = UnionMemberType<'a, C>;
    type UnionMemberTypes = UnionMemberTypes<'a, C>;
    type BaseInputType = BaseInputType<'a, C>;
    type InputType = InputType<'a, C>;
    type BaseOutputType = BaseOutputType<'a, C>;
    type OutputType = OutputType<'a, C>;
    type CustomScalarTypeDefinition = CustomScalarTypeDefinition<'a, C>;
    type ObjectTypeDefinition = ObjectTypeDefinition<'a, C>;
    type InterfaceTypeDefinition = InterfaceTypeDefinition<'a, C>;
    type UnionTypeDefinition = UnionTypeDefinition<'a, C>;
    type InputObjectTypeDefinition = InputObjectTypeDefinition<'a, C>;
    type EnumTypeDefinition = EnumTypeDefinition<'a>;
    type TypeDefinition = TypeDefinition<'a, C>;
    type DirectiveDefinition = DirectiveDefinition<'a, C>;
    type TypeDefinitions<'b> = std::iter::Map<
        Values<'b, &'b str, TypeDefinition<'a, C>>,
        fn(&'b TypeDefinition<'a, C>) -> TypeDefinitionReference<'b, TypeDefinition<'a, C>>
    > where 'a: 'b;
    type DirectiveDefinitions<'b> =
        Values<'b, &'b str, DirectiveDefinition<'a, C>> where 'a: 'b;
    type InterfaceImplementors<'b> = std::iter::Flatten<std::option::IntoIter<std::iter::Map<std::slice::Iter<'b, Arc<ObjectTypeDefinition<'a, C>>>, for<'c> fn(&'c Arc<ObjectTypeDefinition<'a, C>>) -> &'c ObjectTypeDefinition<'a, C>>>> where 'a: 'b;

    fn description(&self) -> Option<&str> {
        self.description.as_ref().map(AsRef::as_ref)
    }

    fn query(&self) -> &Self::ObjectTypeDefinition {
        self.query.as_ref()
    }

    fn mutation(&self) -> Option<&Self::ObjectTypeDefinition> {
        self.mutation.as_deref()
    }

    fn subscription(&self) -> Option<&Self::ObjectTypeDefinition> {
        self.subscription.as_deref()
    }

    fn schema_directives(&self) -> Option<&Self::Directives> {
        self.schema_directives.as_ref()
    }

    fn get_type_definition(
        &self,
        name: &str,
    ) -> Option<TypeDefinitionReference<'_, Self::TypeDefinition>> {
        self.type_definitions.get(name).map(|td| td.as_ref())
    }

    fn type_definitions(&self) -> Self::TypeDefinitions<'_> {
        self.type_definitions.values().map(|td| td.as_ref())
    }

    fn get_directive_definition(&self, name: &str) -> Option<&Self::DirectiveDefinition> {
        self.directive_definitions.get(name)
    }

    fn directive_definitions(&self) -> Self::DirectiveDefinitions<'_> {
        self.directive_definitions.values()
    }

    fn get_interface_implementors(
        &self,
        itd: &Self::InterfaceTypeDefinition,
    ) -> Self::InterfaceImplementors<'_> {
        self.interface_implementors
            .get(itd.name().as_ref())
            .map(|implementors| {
                implementors.iter().map(
                    Arc::as_ref
                        as for<'b> fn(
                            &'b Arc<ObjectTypeDefinition<'a, C>>,
                        ) -> &'b ObjectTypeDefinition<'a, C>,
                )
            })
            .into_iter()
            .flatten()
    }
}
