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
    NewType(NewTypeStruct),
    Struct { id: u32, name: String },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct NewTypeStruct {
    pub id: u32,
    pub name: String,
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

    let e2 = Enum::Struct {
        id: 42,
        name: "Test".to_string(),
    };
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

#[test]
fn test_enum_with_tag_and_content() {
    use switchboard_serde_value::SerdeValue;
    let e1 = EnumWithTagAndContent::Unit;
    let serialized = SerdeValue::serialize_from(&e1).unwrap();
    println!("Serialized EnumWithTagAndContent::Unit: {:?}", serialized);
    let deserialized: EnumWithTagAndContent = SerdeValue::deserialize_into(serialized).unwrap();
    assert_eq!(e1, deserialized);
    let e2 = EnumWithTagAndContent::Struct {
        id: 42,
        name: "Test".to_string(),
    };
    let serialized = SerdeValue::serialize_from(&e2).unwrap();
    println!("Serialized EnumWithTagAndContent::Struct: {:?}", serialized);
    let deserialized: EnumWithTagAndContent = SerdeValue::deserialize_into(serialized).unwrap();
    assert_eq!(e2, deserialized);

    let e3 = EnumWithTagAndContent::Tuple(7, "Hello".to_string());
    let serialized = SerdeValue::serialize_from(&e3).unwrap();
    println!("Serialized EnumWithTagAndContent::Tuple: {:?}", serialized);
    let deserialized: EnumWithTagAndContent = SerdeValue::deserialize_into(serialized).unwrap();
    assert_eq!(e3, deserialized);

    let e4 = EnumWithTagAndContent::NewType(123);
    let serialized = SerdeValue::serialize_from(&e4).unwrap();
    println!(
        "Serialized EnumWithTagAndContent::NewType: {:?}",
        serialized
    );
    let deserialized: EnumWithTagAndContent = SerdeValue::deserialize_into(serialized).unwrap();
    assert_eq!(e4, deserialized);
}

#[test]
fn test_enum_tag_only() {
    use switchboard_serde_value::SerdeValue;
    let e1 = EnumWithTagOnly::Unit;
    let serialized = SerdeValue::serialize_from(&e1).unwrap();
    println!("Serialized EnumWithTagOnly::Unit: {:?}", serialized);
    let deserialized: EnumWithTagOnly = SerdeValue::deserialize_into(serialized).unwrap();
    assert_eq!(e1, deserialized);
    let e2 = EnumWithTagOnly::Struct {
        id: 42,
        name: "Test".to_string(),
    };
    let serialized = SerdeValue::serialize_from(&e2).unwrap();
    println!("Serialized EnumWithTagOnly::Struct: {:?}", serialized);
    let deserialized: EnumWithTagOnly = SerdeValue::deserialize_into(serialized).unwrap();
    assert_eq!(e2, deserialized);

    let e4 = EnumWithTagOnly::NewType(NewTypeStruct {
        id: 1,
        name: "NewType".to_string(),
    });
    let serialized = SerdeValue::serialize_from(&e4).unwrap();
    println!("Serialized EnumWithTagOnly::NewType: {:?}", serialized);
    let deserialized: EnumWithTagOnly = SerdeValue::deserialize_into(serialized).unwrap();
    assert_eq!(e4, deserialized);
}

#[test]
fn test_enum_untagged() {
    use switchboard_serde_value::SerdeValue;
    let e1 = EnumUntagged::Unit;
    let serialized = SerdeValue::serialize_from(&e1).unwrap();
    println!("Serialized EnumUntagged::Unit: {:?}", serialized);
    let deserialized: EnumUntagged = SerdeValue::deserialize_into(serialized).unwrap();
    assert_eq!(e1, deserialized);
    let e2 = EnumUntagged::Struct {
        id: 42,
        name: "Test".to_string(),
    };
    let serialized = SerdeValue::serialize_from(&e2).unwrap();
    println!("Serialized EnumUntagged::Struct: {:?}", serialized);
    let deserialized: EnumUntagged = SerdeValue::deserialize_into(serialized).unwrap();
    assert_eq!(e2, deserialized);

    let e3 = EnumUntagged::Tuple(7, "Hello".to_string());
    let serialized = SerdeValue::serialize_from(&e3).unwrap();
    println!("Serialized EnumUntagged::Tuple: {:?}", serialized);
    let deserialized: EnumUntagged = SerdeValue::deserialize_into(serialized).unwrap();
    assert_eq!(e3, deserialized);

    let e4 = EnumUntagged::NewType(123);
    let serialized = SerdeValue::serialize_from(&e4).unwrap();
    println!("Serialized EnumUntagged::NewType: {:?}", serialized);
    let deserialized: EnumUntagged = SerdeValue::deserialize_into(serialized).unwrap();
    assert_eq!(e4, deserialized);
}
