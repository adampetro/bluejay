use bluejay_parser::ast::definition::{DefinitionDocument, SchemaDefinition};
use bluejay_printer::definition::SchemaDefinitionPrinter;

#[test]
fn test_printer() {
    let s = std::fs::read_to_string("../data/schema.docs.graphql").unwrap();
    let original_document: DefinitionDocument = DefinitionDocument::parse(s.as_str()).unwrap();
    let original_schema_definition = SchemaDefinition::try_from(&original_document).unwrap();

    let printed = SchemaDefinitionPrinter::to_string(&original_schema_definition);
    insta::assert_snapshot!(printed);

    let printed_document: DefinitionDocument = DefinitionDocument::parse(printed.as_str()).unwrap();
    let printed_schema_definition = SchemaDefinition::try_from(&printed_document).unwrap();
    let reprinted = SchemaDefinitionPrinter::to_string(&printed_schema_definition);

    assert_eq!(
        original_document.definition_count(),
        printed_document.definition_count()
    );
    similar_asserts::assert_eq!(printed, reprinted);
}
