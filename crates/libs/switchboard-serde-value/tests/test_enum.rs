#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub enum Enum {
    Unit,
    NewType(i32),
    Tuple(i32, String),
    Struct { id: u32, name: String },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(tag = "type", content = "data")]
pub enum EnumWithTagAndContent {
    Unit,
    NewType(i32),
    Tuple(i32, String),
    Struct { id: u32, name: String },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum EnumWithTagOnly {
    Unit,
    NewType(i32),
    Struct { id: u32, name: String },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(untagged)]
pub enum EnumUntagged {
    Unit,
    NewType(i32),
    Tuple(i32, String),
    Struct { id: u32, name: String },
}

#[test]
fn test_normal_enum() {
    use switchboard_serde_value::SerdeValue;

    let e1 = Enum::Unit;
    let serialized = SerdeValue::serialize_from(&e1).unwrap();
    println!("Serialized Enum::Unit: {:?}", serialized);
    let deserialized: Enum = SerdeValue::deserialize_into(serialized).unwrap();
    assert_eq!(e1, deserialized);

    let e2 = Enum::Struct { id: 42, name: "Test".to_string() };
    let serialized = SerdeValue::serialize_from(&e2).unwrap();
    println!("Serialized Enum::Struct: {:?}", serialized);
    let deserialized: Enum = SerdeValue::deserialize_into(serialized).unwrap();
    assert_eq!(e2, deserialized);

    let e3 = Enum::Tuple(7, "Hello".to_string());
    let serialized = SerdeValue::serialize_from(&e3).unwrap();
    println!("Serialized Enum::Tuple: {:?}", serialized);
    let deserialized: Enum = SerdeValue::deserialize_into(serialized).unwrap();
    assert_eq!(e3, deserialized);

    let e4 = Enum::NewType(123);
    let serialized = SerdeValue::serialize_from(&e4).unwrap();
    println!("Serialized Enum::NewType: {:?}", serialized);
    let deserialized: Enum = SerdeValue::deserialize_into(serialized).unwrap();
    assert_eq!(e4, deserialized);

}