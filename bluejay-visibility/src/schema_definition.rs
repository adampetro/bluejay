use crate::{
    ArgumentsDefinition, BaseInputType, BaseOutputType, Cache, DirectiveDefinition, Directives,
    EnumTypeDefinition, EnumValueDefinition, EnumValueDefinitions, FieldDefinition,
    FieldsDefinition, InputFieldsDefinition, InputObjectTypeDefinition, InputType,
    InputValueDefinition, InputValueDefinitionWithVisibility, InterfaceImplementation,
    InterfaceImplementations, InterfaceTypeDefinition, ObjectTypeDefinition, OutputType,
    ScalarTypeDefinition, TypeDefinition, UnionMemberType, UnionMemberTypes, UnionTypeDefinition,
    Warden,
};
use bluejay_core::definition::{self, prelude::*};
use bluejay_core::AsIter;
use elsa::FrozenMap;
use once_cell::unsync::OnceCell;

pub trait SchemaDefinitionWithVisibility:
    definition::SchemaDefinition<
    InputValueDefinition = <Self as SchemaDefinitionWithVisibility>::InputValueDefinition,
>
{
    type Warden: Warden<SchemaDefinition = Self>;
    type InputValueDefinition: InputValueDefinitionWithVisibility<
        InputType = Self::InputType,
        Directives = Self::Directives,
        Value = Self::Value,
        SchemaDefinition = Self,
    >;
}

pub struct SchemaDefinition<'a, S: SchemaDefinitionWithVisibility> {
    inner: &'a S,
    cache: &'a Cache<'a, S>,
    query: ObjectTypeDefinition<'a, S>,
    mutation: Option<ObjectTypeDefinition<'a, S>>,
    subscription: Option<ObjectTypeDefinition<'a, S>>,
    interface_implementors: FrozenMap<&'a str, Vec<&'a ObjectTypeDefinition<'a, S>>>,
    type_definitions: OnceCell<Vec<&'a TypeDefinition<'a, S>>>,
    directive_definitions: OnceCell<Vec<&'a DirectiveDefinition<'a, S>>>,
    schema_directives: Option<Directives<'a, S>>,
}

impl<'a, S: SchemaDefinitionWithVisibility> SchemaDefinition<'a, S> {
    pub fn new(inner: &'a S, cache: &'a Cache<'a, S>) -> Self {
        Self {
            inner,
            cache,
            query: ObjectTypeDefinition::new(inner.query(), cache),
            mutation: inner
                .mutation()
                .map(|mutation| ObjectTypeDefinition::new(mutation, cache)),
            subscription: inner
                .subscription()
                .map(|subscription| ObjectTypeDefinition::new(subscription, cache)),
            interface_implementors: FrozenMap::new(),
            type_definitions: OnceCell::new(),
            directive_definitions: OnceCell::new(),
            schema_directives: definition::SchemaDefinition::schema_directives(inner)
                .map(|d| Directives::new(d, cache)),
        }
    }

    pub fn inner(&self) -> &'a S {
        self.inner
    }

    pub fn cache(&self) -> &'a Cache<'a, S> {
        self.cache
    }
}

impl<'a, S: SchemaDefinitionWithVisibility + 'a> definition::SchemaDefinition
    for SchemaDefinition<'a, S>
{
    type Value = S::Value;
    type Argument = S::Argument;
    type Arguments = S::Arguments;
    type Directive = S::Directive;
    type Directives = Directives<'a, S>;
    type InputValueDefinition = InputValueDefinition<'a, S>;
    type InputFieldsDefinition = InputFieldsDefinition<'a, S>;
    type ArgumentsDefinition = ArgumentsDefinition<'a, S>;
    type EnumValueDefinition = EnumValueDefinition<'a, S>;
    type EnumValueDefinitions = EnumValueDefinitions<'a, S>;
    type FieldDefinition = FieldDefinition<'a, S>;
    type FieldsDefinition = FieldsDefinition<'a, S>;
    type InterfaceImplementation = InterfaceImplementation<'a, S>;
    type InterfaceImplementations = InterfaceImplementations<'a, S>;
    type UnionMemberType = UnionMemberType<'a, S>;
    type UnionMemberTypes = UnionMemberTypes<'a, S>;
    type BaseInputType = BaseInputType<'a, S>;
    type InputType = InputType<'a, S>;
    type BaseOutputType = BaseOutputType<'a, S>;
    type OutputType = OutputType<'a, S>;
    type CustomScalarTypeDefinition = ScalarTypeDefinition<'a, S>;
    type ObjectTypeDefinition = ObjectTypeDefinition<'a, S>;
    type InterfaceTypeDefinition = InterfaceTypeDefinition<'a, S>;
    type UnionTypeDefinition = UnionTypeDefinition<'a, S>;
    type InputObjectTypeDefinition = InputObjectTypeDefinition<'a, S>;
    type EnumTypeDefinition = EnumTypeDefinition<'a, S>;
    type TypeDefinition = TypeDefinition<'a, S>;
    type DirectiveDefinition = DirectiveDefinition<'a, S>;
    type TypeDefinitions<'b> = std::iter::Map<std::slice::Iter<'b, &'b Self::TypeDefinition>, fn(&&'b Self::TypeDefinition) -> definition::TypeDefinitionReference<'b, Self::TypeDefinition>> where 'a: 'b;
    type DirectiveDefinitions<'b> = std::iter::Copied<std::slice::Iter<'b, &'b Self::DirectiveDefinition>> where 'a: 'b;
    type InterfaceImplementors<'b> = std::iter::Copied<std::slice::Iter<'b, &'b Self::ObjectTypeDefinition>> where 'a: 'b;

    fn description(&self) -> Option<&str> {
        self.inner.description()
    }

    fn query(&self) -> &Self::ObjectTypeDefinition {
        &self.query
    }

    fn mutation(&self) -> Option<&Self::ObjectTypeDefinition> {
        self.mutation.as_ref()
    }

    fn subscription(&self) -> Option<&Self::ObjectTypeDefinition> {
        self.subscription.as_ref()
    }

    fn schema_directives(&self) -> Option<&Self::Directives> {
        self.schema_directives.as_ref()
    }

    fn type_definitions(&self) -> Self::TypeDefinitions<'_> {
        self.type_definitions
            .get_or_init(|| {
                self.inner
                    .type_definitions()
                    .filter_map(|tdr| self.cache.get_or_create_type_definition(tdr))
                    .collect()
            })
            .iter()
            .map(|td| definition::TypeDefinition::as_ref(td))
    }

    fn get_type_definition(
        &self,
        name: &str,
    ) -> Option<definition::TypeDefinitionReference<Self::TypeDefinition>> {
        self.cache
            .get_type_definition(name)
            .or_else(|| {
                self.inner
                    .get_type_definition(name)
                    .and_then(|tdr| self.cache.get_or_create_type_definition(tdr))
            })
            .map(TypeDefinition::as_ref)
    }

    fn get_interface_implementors(
        &self,
        itd: &Self::InterfaceTypeDefinition,
    ) -> Self::InterfaceImplementors<'_> {
        self.interface_implementors
            .get(itd.name())
            .map(|ii| ii.iter())
            .unwrap_or_else(|| {
                let interface_implementors = self
                    .inner
                    .get_interface_implementors(itd.inner())
                    .filter_map(|otd| {
                        let otd = self
                            .cache
                            .get_or_create_type_definition(
                                definition::TypeDefinitionReference::Object(otd),
                            )?
                            .as_object()
                            .unwrap();

                        otd.interface_implementations()
                            .map_or(false, |interface_implementations| {
                                interface_implementations
                                    .iter()
                                    .any(|ii| ii.interface().name() == itd.name())
                            })
                            .then_some(otd)
                    })
                    .collect();
                self.interface_implementors
                    .insert(itd.inner().name(), interface_implementors)
                    .iter()
            })
            .copied()
    }

    fn directive_definitions(&self) -> Self::DirectiveDefinitions<'_> {
        self.directive_definitions
            .get_or_init(|| {
                self.inner
                    .directive_definitions()
                    .map(|dd| self.cache.get_or_create_directive_definition(dd))
                    .collect()
            })
            .iter()
            .copied()
    }

    fn get_directive_definition(&self, name: &str) -> Option<&Self::DirectiveDefinition> {
        self.cache.get_directive_definition(name).or_else(|| {
            self.inner
                .get_directive_definition(name)
                .map(|dd| self.cache.get_or_create_directive_definition(dd))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Cache, InputValueDefinitionWithVisibility, SchemaDefinition,
        SchemaDefinitionWithVisibility, Warden,
    };
    use bluejay_core::{definition::prelude::*, AsIter};
    use bluejay_parser::ast::{
        definition::{
            Context, DefaultContext, DefinitionDocument, InputValueDefinition,
            SchemaDefinition as ParserSchemaDefinition,
        },
        Directives,
    };
    use bluejay_printer::definition::DisplaySchemaDefinition;
    use std::marker::PhantomData;

    impl<'a, C: Context> InputValueDefinitionWithVisibility for InputValueDefinition<'a, C> {
        type SchemaDefinition = ParserSchemaDefinition<'a, C>;
    }

    impl<'a, C: Context> SchemaDefinitionWithVisibility for ParserSchemaDefinition<'a, C> {
        type Warden = DirectiveWarden<'a, C>;
        type InputValueDefinition = InputValueDefinition<'a, C>;
    }

    pub struct DirectiveWarden<'a, C: Context = DefaultContext>(
        PhantomData<ParserSchemaDefinition<'a, C>>,
    );

    impl<'a, C: Context> DirectiveWarden<'a, C> {
        fn has_visible_directive(directives: Option<&Directives<'a, true>>) -> bool {
            directives.map_or(false, |directives| {
                directives
                    .iter()
                    .any(|directive| directive.name() == "visible")
            })
        }

        fn new() -> Self {
            Self(PhantomData)
        }
    }

    impl<'a, C: Context> Warden for DirectiveWarden<'a, C> {
        type SchemaDefinition = ParserSchemaDefinition<'a, C>;

        fn is_enum_value_definition_visible(
            &self,
            enum_value_definition: &<Self::SchemaDefinition as bluejay_core::definition::SchemaDefinition>::EnumValueDefinition,
        ) -> bool {
            Self::has_visible_directive(enum_value_definition.directives())
        }

        fn is_field_definition_visible(
            &self,
            field_definition: &<Self::SchemaDefinition as bluejay_core::definition::SchemaDefinition>::FieldDefinition,
        ) -> bool {
            Self::has_visible_directive(field_definition.directives())
        }

        fn is_input_value_definition_visible(
            &self,
            input_value_definition: &<Self::SchemaDefinition as bluejay_core::definition::SchemaDefinition>::InputValueDefinition,
        ) -> bool {
            Self::has_visible_directive(input_value_definition.directives())
        }

        fn is_interface_implementation_visible(
            &self,
            _: &<Self::SchemaDefinition as bluejay_core::definition::SchemaDefinition>::InterfaceImplementation,
        ) -> bool {
            true
        }

        fn is_union_member_type_visible(
            &self,
            _: &<Self::SchemaDefinition as bluejay_core::definition::SchemaDefinition>::UnionMemberType,
        ) -> bool {
            true
        }

        fn is_directive_definition_visible(
            &self,
            _: &<Self::SchemaDefinition as bluejay_core::definition::SchemaDefinition>::DirectiveDefinition,
        ) -> bool {
            true
        }

        fn is_custom_scalar_type_definition_visible(
            &self,
            custom_scalar_type_definition: &<Self::SchemaDefinition as bluejay_core::definition::SchemaDefinition>::CustomScalarTypeDefinition,
        ) -> bool {
            Self::has_visible_directive(custom_scalar_type_definition.directives())
        }

        fn is_enum_type_definition_visible(
            &self,
            enum_type_definition: &<Self::SchemaDefinition as bluejay_core::definition::SchemaDefinition>::EnumTypeDefinition,
        ) -> bool {
            Self::has_visible_directive(enum_type_definition.directives())
        }

        fn is_input_object_type_definition_visible(
            &self,
            input_object_type_definition: &<Self::SchemaDefinition as bluejay_core::definition::SchemaDefinition>::InputObjectTypeDefinition,
        ) -> bool {
            Self::has_visible_directive(input_object_type_definition.directives())
        }

        fn is_interface_type_definition_visible(
            &self,
            interface_type_definition: &<Self::SchemaDefinition as bluejay_core::definition::SchemaDefinition>::InterfaceTypeDefinition,
        ) -> bool {
            Self::has_visible_directive(interface_type_definition.directives())
        }

        fn is_object_type_definition_visible(
            &self,
            object_type_definition: &<Self::SchemaDefinition as bluejay_core::definition::SchemaDefinition>::ObjectTypeDefinition,
        ) -> bool {
            Self::has_visible_directive(object_type_definition.directives())
        }

        fn is_union_type_definition_visible(
            &self,
            union_type_definition: &<Self::SchemaDefinition as bluejay_core::definition::SchemaDefinition>::UnionTypeDefinition,
        ) -> bool {
            Self::has_visible_directive(union_type_definition.directives())
        }
    }

    #[test]
    fn test_visibility() {
        let path = std::path::Path::new("src/test_data/schema.graphql");
        let input = std::fs::read_to_string(path).unwrap();
        let definition_document: DefinitionDocument = DefinitionDocument::parse(&input)
            .unwrap_or_else(|_| panic!("Schema `{}` had parse errors", path.display()));
        let schema_definition = ParserSchemaDefinition::try_from(&definition_document)
            .unwrap_or_else(|_| panic!("Schema `{}` had coercion errors", path.display()));

        let cache = Cache::new(DirectiveWarden::new());
        let visibility_scoped_schema_definition = SchemaDefinition::new(&schema_definition, &cache);

        let printed_schema_definition =
            DisplaySchemaDefinition::to_string(&visibility_scoped_schema_definition);

        insta::assert_snapshot!(printed_schema_definition);
    }
}
