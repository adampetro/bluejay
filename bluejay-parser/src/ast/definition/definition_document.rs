use crate::ast::definition::{
    BaseInputType, BaseOutputType, Context, CustomScalarTypeDefinition, DefaultContext,
    DirectiveDefinition, EnumTypeDefinition, ExplicitSchemaDefinition, FieldsDefinition,
    InputObjectTypeDefinition, InputValueDefinition, InterfaceImplementations,
    InterfaceTypeDefinition, ObjectTypeDefinition, SchemaDefinition, TypeDefinition,
    UnionTypeDefinition,
};
use crate::ast::{FromTokens, ParseError, ScannerTokens, Tokens};
use crate::scanner::LogosScanner;
use crate::Error;
use bluejay_core::definition::{
    DirectiveDefinition as CoreDirectiveDefinition, FieldDefinition as CoreFieldDefinition,
    InputObjectTypeDefinition as CoreInputObjectTypeDefinition,
    InputValueDefinition as CoreInputValueDefinition,
    InterfaceTypeDefinition as CoreInterfaceTypeDefinition,
    ObjectTypeDefinition as CoreObjectTypeDefinition, OutputType,
    TypeDefinition as CoreTypeDefinition, UnionTypeDefinition as CoreUnionTypeDefinition,
};
use bluejay_core::{AsIter, BuiltinScalarDefinition, IntoEnumIterator, OperationType};
use std::collections::btree_map::Entry;
use std::collections::{BTreeMap, HashMap, HashSet};

mod definition_document_error;
use definition_document_error::DefinitionDocumentError;

#[derive(Debug)]
pub struct DefinitionDocument<'a, C: Context = DefaultContext> {
    schema_definitions: Vec<ExplicitSchemaDefinition<'a>>,
    directive_definitions: Vec<DirectiveDefinition<'a, C>>,
    type_definitions: Vec<TypeDefinition<'a, C>>,
}

#[derive(Debug)]
pub struct ImplicitSchemaDefinition<'a, C: Context> {
    query: &'a ObjectTypeDefinition<'a, C>,
    mutation: Option<&'a ObjectTypeDefinition<'a, C>>,
    subscription: Option<&'a ObjectTypeDefinition<'a, C>>,
}

type ExplicitSchemaDefinitionWithRootTypes<'a, C> = (
    &'a ExplicitSchemaDefinition<'a>,
    &'a ObjectTypeDefinition<'a, C>,
    Option<&'a ObjectTypeDefinition<'a, C>>,
    Option<&'a ObjectTypeDefinition<'a, C>>,
);

impl<'a, C: Context> DefinitionDocument<'a, C> {
    fn new() -> Self {
        Self {
            schema_definitions: Vec::new(),
            directive_definitions: vec![
                DirectiveDefinition::skip(),
                DirectiveDefinition::include(),
                DirectiveDefinition::deprecated(),
                DirectiveDefinition::specified_by(),
            ],
            type_definitions: vec![
                ObjectTypeDefinition::__schema().into(),
                ObjectTypeDefinition::__type().into(),
                ObjectTypeDefinition::__field().into(),
                ObjectTypeDefinition::__input_value().into(),
                ObjectTypeDefinition::__enum_value().into(),
                ObjectTypeDefinition::__directive().into(),
                EnumTypeDefinition::__type_kind().into(),
                EnumTypeDefinition::__directive_location().into(),
            ],
        }
    }

    fn parse_definition<'b, S, T: FromTokens<'b> + Into<S>>(
        definitions: &mut Vec<S>,
        tokens: &mut impl Tokens<'b>,
        errors: &mut Vec<ParseError>,
        last_pass_had_error: &mut bool,
    ) {
        match T::from_tokens(tokens) {
            Ok(definition) => {
                definitions.push(definition.into());
                *last_pass_had_error = false;
            }
            Err(err) => {
                if !*last_pass_had_error {
                    errors.push(err);
                    *last_pass_had_error = true;
                }
            }
        }
    }

    pub fn parse(s: &'a str) -> Result<Self, Vec<Error>> {
        let scanner = LogosScanner::new(s);
        let mut tokens = ScannerTokens::new(scanner);

        let mut instance: Self = Self::new();
        let mut errors = Vec::new();
        let mut last_pass_had_error = false;

        loop {
            match Self::next_definition_identifier(&mut tokens) {
                Some(CustomScalarTypeDefinition::<C>::SCALAR_IDENTIFIER) => {
                    Self::parse_definition::<_, CustomScalarTypeDefinition<C>>(
                        &mut instance.type_definitions,
                        &mut tokens,
                        &mut errors,
                        &mut last_pass_had_error,
                    )
                }
                Some(ObjectTypeDefinition::<C>::TYPE_IDENTIFIER) => {
                    Self::parse_definition::<_, ObjectTypeDefinition<C>>(
                        &mut instance.type_definitions,
                        &mut tokens,
                        &mut errors,
                        &mut last_pass_had_error,
                    )
                }
                Some(InputObjectTypeDefinition::<C>::INPUT_IDENTIFIER) => {
                    Self::parse_definition::<_, InputObjectTypeDefinition<C>>(
                        &mut instance.type_definitions,
                        &mut tokens,
                        &mut errors,
                        &mut last_pass_had_error,
                    )
                }
                Some(EnumTypeDefinition::ENUM_IDENTIFIER) => {
                    Self::parse_definition::<_, EnumTypeDefinition>(
                        &mut instance.type_definitions,
                        &mut tokens,
                        &mut errors,
                        &mut last_pass_had_error,
                    )
                }
                Some(UnionTypeDefinition::<C>::UNION_IDENTIFIER) => {
                    Self::parse_definition::<_, UnionTypeDefinition<C>>(
                        &mut instance.type_definitions,
                        &mut tokens,
                        &mut errors,
                        &mut last_pass_had_error,
                    )
                }
                Some(InterfaceTypeDefinition::<C>::INTERFACE_IDENTIFIER) => {
                    Self::parse_definition::<_, InterfaceTypeDefinition<C>>(
                        &mut instance.type_definitions,
                        &mut tokens,
                        &mut errors,
                        &mut last_pass_had_error,
                    )
                }
                Some(ExplicitSchemaDefinition::SCHEMA_IDENTIFIER) => {
                    Self::parse_definition::<_, ExplicitSchemaDefinition>(
                        &mut instance.schema_definitions,
                        &mut tokens,
                        &mut errors,
                        &mut last_pass_had_error,
                    )
                }
                Some(DirectiveDefinition::<C>::DIRECTIVE_IDENTIFIER) => {
                    Self::parse_definition::<_, DirectiveDefinition<C>>(
                        &mut instance.directive_definitions,
                        &mut tokens,
                        &mut errors,
                        &mut last_pass_had_error,
                    )
                }
                _ => {
                    if let Some(token) = tokens.next() {
                        if !last_pass_had_error {
                            errors.push(ParseError::UnexpectedToken { span: token.into() });
                            last_pass_had_error = true;
                        }
                    } else {
                        break;
                    }
                }
            }
        }

        let errors = if tokens.errors.is_empty() {
            if errors.is_empty() && instance.is_empty() {
                vec![ParseError::EmptyDocument.into()]
            } else {
                errors.into_iter().map(Into::into).collect()
            }
        } else {
            tokens.errors.into_iter().map(Into::into).collect()
        };

        if errors.is_empty() {
            instance.insert_builtin_scalar_definitions();
            instance.add_query_root_fields();
            Ok(instance)
        } else {
            Err(errors)
        }
    }

    /// Inserts builtin scalars only for type names that have not already been parsed
    /// to allow overriding of builtin scalars
    fn insert_builtin_scalar_definitions(&mut self) {
        let mut builtin_scalars_by_name: HashMap<&str, BuiltinScalarDefinition> =
            HashMap::from_iter(BuiltinScalarDefinition::iter().map(|bstd| (bstd.name(), bstd)));

        self.type_definitions.iter().for_each(|td| {
            builtin_scalars_by_name.remove(td.name());
        });

        self.type_definitions.extend(
            builtin_scalars_by_name
                .into_values()
                .map(TypeDefinition::BuiltinScalar),
        );
    }

    fn add_query_root_fields(&mut self) {
        let explicit_query_roots: HashSet<&str> = HashSet::from_iter(
            self.schema_definitions
                .iter()
                .flat_map(|schema_definition| {
                    schema_definition
                        .root_operation_type_definitions()
                        .iter()
                        .filter_map(|rotd| {
                            (rotd.operation_type() == OperationType::Query).then(|| rotd.name())
                        })
                }),
        );

        self.type_definitions
            .iter_mut()
            .for_each(|type_definition| {
                if let TypeDefinition::Object(otd) = type_definition {
                    let name = otd.name().as_ref();
                    if name == "Query" || explicit_query_roots.contains(name) {
                        otd.add_query_root_fields();
                    }
                }
            })
    }

    fn is_empty(&self) -> bool {
        self.definition_count() == 0
    }

    fn next_definition_identifier(tokens: &mut impl Tokens<'a>) -> Option<&str> {
        let idx_to_peek = if tokens.peek_string_value(0) { 1 } else { 0 };
        tokens.peek_name(idx_to_peek).map(AsRef::as_ref)
    }

    pub fn definition_count(&self) -> usize {
        self.directive_definitions
            .iter()
            .filter(|dd| !dd.is_builtin())
            .count()
            + self.schema_definitions.len()
            + self
                .type_definitions
                .iter()
                .filter(|td| !td.as_ref().is_builtin())
                .count()
    }

    pub fn directive_definitions(&self) -> &[DirectiveDefinition<'a, C>] {
        &self.directive_definitions
    }

    fn index_directive_definitions(
        &'a self,
        errors: &mut Vec<DefinitionDocumentError<'a, C>>,
    ) -> BTreeMap<&str, &'a DirectiveDefinition<'a, C>> {
        let mut indexed: BTreeMap<&str, &DirectiveDefinition<'a, C>> = BTreeMap::new();
        let mut duplicates: BTreeMap<&str, Vec<&DirectiveDefinition<'a, C>>> = BTreeMap::new();

        self.directive_definitions
            .iter()
            .for_each(|directive_definition| {
                match indexed.entry(directive_definition.name().as_ref()) {
                    Entry::Vacant(entry) => {
                        entry.insert(directive_definition);
                    }
                    Entry::Occupied(entry) => {
                        duplicates
                            .entry(directive_definition.name().as_ref())
                            .or_insert_with(|| vec![entry.get()])
                            .push(directive_definition);
                    }
                }
            });

        errors.extend(duplicates.into_iter().map(|(name, definitions)| {
            DefinitionDocumentError::DuplicateDirectiveDefinitions { name, definitions }
        }));

        indexed
    }

    fn index_type_definitions(
        &'a self,
        errors: &mut Vec<DefinitionDocumentError<'a, C>>,
    ) -> BTreeMap<&str, &TypeDefinition<'a, C>> {
        let mut indexed: BTreeMap<&str, &TypeDefinition<'a, C>> = BTreeMap::new();
        let mut duplicates: BTreeMap<&str, Vec<&TypeDefinition<'a, C>>> = BTreeMap::new();

        self.type_definitions
            .iter()
            .for_each(|td| match indexed.entry(td.name()) {
                Entry::Vacant(entry) => {
                    entry.insert(td);
                }
                Entry::Occupied(entry) => {
                    duplicates
                        .entry(td.name())
                        .or_insert_with(|| vec![entry.get()])
                        .push(td);
                }
            });

        errors.extend(duplicates.into_iter().map(|(name, definitions)| {
            DefinitionDocumentError::DuplicateTypeDefinitions { name, definitions }
        }));

        indexed
    }

    fn implicit_schema_definition(
        indexed_type_definitions: &BTreeMap<&str, &'a TypeDefinition<'a, C>>,
    ) -> Result<Option<ImplicitSchemaDefinition<'a, C>>, Vec<DefinitionDocumentError<'a, C>>> {
        let mut errors = Vec::new();
        let query =
            Self::implicit_root_operation_type("Query", indexed_type_definitions, &mut errors);
        let mutation =
            Self::implicit_root_operation_type("Mutation", indexed_type_definitions, &mut errors);
        let subscription = Self::implicit_root_operation_type(
            "Subscription",
            indexed_type_definitions,
            &mut errors,
        );

        if !errors.is_empty() {
            return Err(errors);
        }

        if let Some(query) = query {
            Ok(Some(ImplicitSchemaDefinition {
                query,
                mutation,
                subscription,
            }))
        } else if mutation.is_some() || subscription.is_some() {
            Err(vec![
                DefinitionDocumentError::ImplicitSchemaDefinitionMissingQuery,
            ])
        } else {
            Ok(None)
        }
    }

    fn implicit_root_operation_type(
        name: &str,
        indexed_type_definitions: &BTreeMap<&str, &'a TypeDefinition<'a, C>>,
        errors: &mut Vec<DefinitionDocumentError<'a, C>>,
    ) -> Option<&'a ObjectTypeDefinition<'a, C>> {
        match indexed_type_definitions.get(name) {
            Some(TypeDefinition::Object(o)) => Some(o),
            Some(definition) => {
                errors.push(
                    DefinitionDocumentError::ImplicitRootOperationTypeNotAnObject { definition },
                );
                None
            }
            None => None,
        }
    }

    fn explicit_schema_definition(
        &'a self,
        indexed_type_definitions: &BTreeMap<&str, &'a TypeDefinition<'a, C>>,
    ) -> Result<
        Option<ExplicitSchemaDefinitionWithRootTypes<'a, C>>,
        Vec<DefinitionDocumentError<'a, C>>,
    > {
        let mut errors = Vec::new();
        if let Some(first) = self.schema_definitions.first() {
            if self.schema_definitions.len() == 1 {
                let query = match Self::explicit_operation_type_definition(
                    OperationType::Query,
                    first,
                    indexed_type_definitions,
                ) {
                    Ok(query) => query,
                    Err(err) => {
                        errors.push(err);
                        None
                    }
                };
                let mutation = match Self::explicit_operation_type_definition(
                    OperationType::Mutation,
                    first,
                    indexed_type_definitions,
                ) {
                    Ok(mutation) => mutation,
                    Err(err) => {
                        errors.push(err);
                        None
                    }
                };
                let subscription = match Self::explicit_operation_type_definition(
                    OperationType::Subscription,
                    first,
                    indexed_type_definitions,
                ) {
                    Ok(subscription) => subscription,
                    Err(err) => {
                        errors.push(err);
                        None
                    }
                };
                if !errors.is_empty() {
                    return Err(errors);
                }
                if let Some(query) = query {
                    Ok(Some((first, query, mutation, subscription)))
                } else {
                    Err(vec![
                        DefinitionDocumentError::ExplicitSchemaDefinitionMissingQuery {
                            definition: first,
                        },
                    ])
                }
            } else {
                Err(vec![
                    DefinitionDocumentError::DuplicateExplicitSchemaDefinitions {
                        definitions: &self.schema_definitions,
                    },
                ])
            }
        } else {
            Ok(None)
        }
    }

    fn explicit_operation_type_definition(
        operation_type: OperationType,
        explicit_schema_definition: &'a ExplicitSchemaDefinition<'a>,
        indexed_type_definitions: &BTreeMap<&str, &'a TypeDefinition<'a, C>>,
    ) -> Result<Option<&'a ObjectTypeDefinition<'a, C>>, DefinitionDocumentError<'a, C>> {
        let root_operation_type_definitions: Vec<_> = explicit_schema_definition
            .root_operation_type_definitions()
            .iter()
            .filter(|rotd| rotd.operation_type() == operation_type)
            .collect();

        if let Some(first) = root_operation_type_definitions.first() {
            if root_operation_type_definitions.len() == 1 {
                match indexed_type_definitions.get(first.name()) {
                    Some(TypeDefinition::Object(o)) => Ok(Some(o)),
                    Some(_) => Err(
                        DefinitionDocumentError::ExplicitRootOperationTypeNotAnObject {
                            name: first.name_token(),
                        },
                    ),
                    None => Err(
                        DefinitionDocumentError::ExplicitRootOperationTypeDoesNotExist {
                            root_operation_type_definition: first,
                        },
                    ),
                }
            } else {
                Err(
                    DefinitionDocumentError::DuplicateExplicitRootOperationDefinitions {
                        operation_type,
                        root_operation_type_definitions,
                    },
                )
            }
        } else {
            Ok(None)
        }
    }

    fn resolve_type_definitions(
        indexed_type_definitions: &BTreeMap<&str, &'a TypeDefinition<'a, C>>,
        indexed_directive_definitions: &BTreeMap<&str, &'a DirectiveDefinition<'a, C>>,
        errors: &mut Vec<DefinitionDocumentError<'a, C>>,
    ) {
        indexed_type_definitions
            .values()
            .for_each(|type_definition| match type_definition {
                TypeDefinition::Object(otd) => {
                    Self::resolve_fields_definition_types(
                        indexed_type_definitions,
                        otd.fields_definition(),
                        errors,
                    );
                    if let Some(interface_implementations) = otd.interface_implementations() {
                        Self::resolve_interface_implementations(
                            indexed_type_definitions,
                            interface_implementations,
                            errors,
                        );
                    }
                }
                TypeDefinition::Interface(itd) => {
                    Self::resolve_fields_definition_types(
                        indexed_type_definitions,
                        itd.fields_definition(),
                        errors,
                    );
                    if let Some(interface_implementations) = itd.interface_implementations() {
                        Self::resolve_interface_implementations(
                            indexed_type_definitions,
                            interface_implementations,
                            errors,
                        );
                    }
                }
                TypeDefinition::Union(utd) => {
                    Self::resolve_fields_definition_types(
                        indexed_type_definitions,
                        utd.fields_definition(),
                        errors,
                    );
                    utd.union_member_types().iter().for_each(|member_type| {
                        match indexed_type_definitions.get(member_type.name().as_ref()) {
                            Some(TypeDefinition::Object(otd)) => {
                                member_type.set_type(otd).unwrap();
                            }
                            Some(_) => errors.push(
                                DefinitionDocumentError::ReferencedUnionMemberTypeIsNotAnObject {
                                    name: member_type.name(),
                                },
                            ),
                            None => {
                                errors.push(DefinitionDocumentError::ReferencedTypeDoesNotExist {
                                    name: member_type.name(),
                                })
                            }
                        }
                    });
                }
                TypeDefinition::InputObject(iotd) => Self::resolve_input_types(
                    indexed_type_definitions,
                    iotd.input_field_definitions().iter(),
                    errors,
                ),
                TypeDefinition::BuiltinScalar(_)
                | TypeDefinition::CustomScalar(_)
                | TypeDefinition::Enum(_) => {}
            });

        indexed_directive_definitions
            .values()
            .for_each(|directive_definition| {
                if let Some(arguments_definition) = directive_definition.arguments_definition() {
                    Self::resolve_input_types(
                        indexed_type_definitions,
                        arguments_definition.iter(),
                        errors,
                    );
                }
            })
    }

    fn resolve_fields_definition_types(
        indexed_type_definitions: &BTreeMap<&str, &'a TypeDefinition<'a, C>>,
        fields_definition: &'a FieldsDefinition<'a, C>,
        errors: &mut Vec<DefinitionDocumentError<'a, C>>,
    ) {
        fields_definition.iter().for_each(|field_definition| {
            let t = field_definition.r#type().as_ref().base();
            match indexed_type_definitions.get(t.name().as_ref()) {
                Some(&td) => match BaseOutputType::core_type_from_type_definition(td) {
                    Ok(core_t) => t.set_type(core_t).unwrap(),
                    Err(_) => {
                        errors.push(DefinitionDocumentError::ReferencedTypeIsNotAnOutputType {
                            name: t.name(),
                        })
                    }
                },
                None => errors
                    .push(DefinitionDocumentError::ReferencedTypeDoesNotExist { name: t.name() }),
            }

            if let Some(arguments_definition) = field_definition.arguments_definition() {
                Self::resolve_input_types(
                    indexed_type_definitions,
                    arguments_definition.iter(),
                    errors,
                )
            }
        })
    }

    fn resolve_interface_implementations(
        indexed_type_definitions: &BTreeMap<&str, &'a TypeDefinition<'a, C>>,
        interface_impelementations: &'a InterfaceImplementations<'a, C>,
        errors: &mut Vec<DefinitionDocumentError<'a, C>>,
    ) {
        interface_impelementations
            .iter()
            .for_each(|interface_implementation| {
                let name = interface_implementation.interface_name();
                match indexed_type_definitions.get(name.as_ref()) {
                    Some(TypeDefinition::Interface(itd)) => {
                        interface_implementation.set_type(itd).unwrap();
                    }
                    Some(_) => errors
                        .push(DefinitionDocumentError::ReferencedTypeIsNotAnInterface { name }),
                    None => {
                        errors.push(DefinitionDocumentError::ReferencedTypeDoesNotExist { name })
                    }
                }
            })
    }

    fn resolve_input_types(
        indexed_type_definitions: &BTreeMap<&str, &'a TypeDefinition<'a, C>>,
        input_value_definitions: impl Iterator<Item = &'a InputValueDefinition<'a, C>>,
        errors: &mut Vec<DefinitionDocumentError<'a, C>>,
    ) {
        input_value_definitions.for_each(|input_value_definition| {
            let t = input_value_definition.r#type().base();
            match indexed_type_definitions.get(t.name().as_ref()) {
                Some(&td) => match BaseInputType::core_type_from_type_definition(td) {
                    Ok(core_t) => t.set_type(core_t).unwrap(),
                    Err(_) => {
                        errors.push(DefinitionDocumentError::ReferencedTypeIsNotAnInputType {
                            name: t.name(),
                        })
                    }
                },
                None => errors
                    .push(DefinitionDocumentError::ReferencedTypeDoesNotExist { name: t.name() }),
            }
        })
    }
}

impl<'a, C: Context> TryFrom<&'a DefinitionDocument<'a, C>> for SchemaDefinition<'a, C> {
    type Error = Vec<DefinitionDocumentError<'a, C>>;

    fn try_from(definition_document: &'a DefinitionDocument<'a, C>) -> Result<Self, Self::Error> {
        let mut errors = Vec::new();

        let indexed_type_definitions = definition_document.index_type_definitions(&mut errors);

        let indexed_directive_definitions =
            definition_document.index_directive_definitions(&mut errors);

        DefinitionDocument::resolve_type_definitions(
            &indexed_type_definitions,
            &indexed_directive_definitions,
            &mut errors,
        );

        if !errors.is_empty() {
            return Err(errors);
        }

        if let Some((explicit, query, mutation, subscription)) =
            definition_document.explicit_schema_definition(&indexed_type_definitions)?
        {
            return Ok(Self::new(
                indexed_type_definitions,
                indexed_directive_definitions,
                explicit.description(),
                query,
                mutation,
                subscription,
                explicit.directives(),
            ));
        }

        match DefinitionDocument::implicit_schema_definition(&indexed_type_definitions)? {
            Some(implicit) => Ok(Self::new(
                indexed_type_definitions,
                indexed_directive_definitions,
                None,
                implicit.query,
                implicit.mutation,
                implicit.subscription,
                None,
            )),
            None => Err(vec![DefinitionDocumentError::NoSchemaDefinition]),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use bluejay_core::{
        definition::{
            DirectiveDefinition, FieldDefinition as CoreFieldDefinition,
            ObjectTypeDefinition as CoreObjectTypeDefinition,
            SchemaDefinition as CoreSchemaDefinition,
        },
        AsIter,
    };

    use super::{DefinitionDocument, SchemaDefinition};

    #[test]
    fn smoke_test() {
        let s = r#"
        """
        Description
        """
        type Object {
            foo: String!
        }
        "#;

        let document: DefinitionDocument = DefinitionDocument::parse(s).unwrap();

        assert_eq!(1, document.definition_count());
    }

    #[test]
    fn builtin_fields_and_types_test() {
        let s = r#"
        type Query {
            foo: String!
        }

        type Mutation {
            foo: String!
        }
        "#;

        let document: DefinitionDocument =
            DefinitionDocument::parse(s).expect("Document had parse errors");

        let schema_definition = SchemaDefinition::try_from(&document)
            .expect("Could not convert document to schema definition");

        let query_root_builtin_fields: HashSet<&str> = schema_definition
            .query()
            .fields_definition()
            .iter()
            .filter_map(|fd| fd.is_builtin().then_some(fd.name()))
            .collect();

        assert_eq!(
            HashSet::from(["__typename", "__schema", "__type"]),
            query_root_builtin_fields,
        );

        let mutation_root = schema_definition
            .mutation()
            .expect("Schema definition did not have a mutation root");

        let mutation_root_builtin_fields: HashSet<&str> = mutation_root
            .fields_definition()
            .iter()
            .filter_map(|fd| fd.is_builtin().then_some(fd.name()))
            .collect();

        assert_eq!(HashSet::from(["__typename"]), mutation_root_builtin_fields);

        let builtin_directives: HashSet<&str> = schema_definition
            .directive_definitions()
            .filter_map(|dd| dd.is_builtin().then_some(dd.name()))
            .collect();

        assert_eq!(
            HashSet::from(["include", "skip", "deprecated", "specifiedBy"]),
            builtin_directives
        );

        let builtin_types: HashSet<&str> = schema_definition
            .type_definitions()
            .filter_map(|td| td.is_builtin().then_some(td.name()))
            .collect();

        assert_eq!(
            HashSet::from([
                "__TypeKind",
                "__DirectiveLocation",
                "__Schema",
                "__Type",
                "__Field",
                "__InputValue",
                "__EnumValue",
                "__Directive",
                "String",
                "ID",
                "Boolean",
                "Int",
                "Float",
            ]),
            builtin_types,
        );
    }
}
