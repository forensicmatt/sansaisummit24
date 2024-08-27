use serde_json::json;
use evtx_clustering::transformer::{DocumentTransformer, FieldRetriever};


#[test]
fn test_field_retriever() {
    let field_retriever = FieldRetriever::from_pattern("field1", "test")
        .expect("Could not compile.");

    let value = field_retriever.search(json!({"test": "YAY"})).unwrap();
    assert_eq!(json!(value), json!("YAY"));

    let value = field_retriever.get_map(json!({"test": "YAY"})).unwrap();
    assert_eq!(json!(value), json!({"field1": "YAY"}));
}


#[test]
fn test_document_transformer() {
    let doc_transformer = DocumentTransformer::empty()
        .add_field_from_pattern("field1", "test").unwrap()
        .add_field_from_pattern("field2", "'BLAH'").unwrap();

    let value = doc_transformer.get_map(json!({"test": "YAY"})).unwrap();
    assert_eq!(json!(value), json!({"field1": "YAY", "field2": "BLAH"}));
}