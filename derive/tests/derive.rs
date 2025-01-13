use postchain_client_derive::StructMetadata;
use bigdecimal::BigDecimal;
use num_bigint::BigInt;

pub trait StructMetadata {
    fn field_names_and_types() -> std::collections::BTreeMap<String, String>;
}

#[allow(dead_code)]
#[derive(StructMetadata)]
struct TestStruct2 {
    text: String
}


#[allow(dead_code)]
#[derive(StructMetadata)]
struct TestStruct {
    text: String,
    int: i64,
    bigdecimal: BigDecimal,
    bigint: BigInt,
    nested_struct: TestStruct2,
}

#[test]
fn test_struct_metadata() {
    let fields = TestStruct::field_names_and_types();
    assert_eq!(fields.get("bigdecimal"), Some(&"BigDecimal".to_string()));
    assert_eq!(fields.get("bigint"), Some(&"BigInt".to_string()));
    assert_eq!(fields.get("nested_struct"), Some(&"TestStruct2".to_string()));
}