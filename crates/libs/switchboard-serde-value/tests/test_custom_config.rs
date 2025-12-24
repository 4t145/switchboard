use switchboard_serde_value::*;

#[derive(Debug, serde::Serialize, serde::Deserialize, bincode::Encode, bincode::Decode)]
pub struct ConfigWithCustomField {
    pub name: String,
    pub inline_config: LinkOrMap,
    pub linked_config: LinkOrMap,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, bincode::Encode, bincode::Decode)]
#[serde(untagged)]
pub enum LinkOrMap {
    Link(String),
    Map(SerdeMap),
}

impl LinkOrMap {
    pub fn is_link(&self) -> bool {
        matches!(self, LinkOrMap::Link(_))
    }

    pub fn is_map(&self) -> bool {
        matches!(self, LinkOrMap::Map(_))
    }
}

fn resource(path: &str) -> String {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    format!("{}/tests/test_custom_config/{}", manifest_dir, path)
}
#[test]
fn test_custom_config_json() {
    let data = std::fs::read_to_string(resource("config.json")).unwrap();
    let config: ConfigWithCustomField = serde_json::from_str(&data).unwrap();
    assert_eq!(config.name, "json");
    assert!(config.inline_config.is_map());
    assert!(config.linked_config.is_link());
}

#[test]
fn test_custom_config_toml() {
    let data = std::fs::read_to_string(resource("config.toml")).unwrap();

    let config: ConfigWithCustomField = toml::from_str(&data).unwrap();
    assert_eq!(config.name, "toml");
    assert!(config.inline_config.is_map());
    assert!(config.linked_config.is_link());
}

#[test]
fn test_custom_config_bincode() {
    let config = ConfigWithCustomField {
        name: "bincode".to_string(),
        inline_config: LinkOrMap::Map(SerdeMap(vec![
            (
                SerdeValue::String("key1".to_string()),
                SerdeValue::Primitive(SerdePrimitive::U64(42)),
            ),
            (
                SerdeValue::String("key2".to_string()),
                SerdeValue::Primitive(SerdePrimitive::Bool(true)),
            ),
        ])),
        linked_config: LinkOrMap::Link("http://example.com/config".to_string()),
    };
    let encoded = bincode::encode_to_vec(&config, bincode::config::standard()).unwrap();
    let decoded_config: ConfigWithCustomField =
        bincode::decode_from_slice(&encoded, bincode::config::standard())
            .unwrap()
            .0;
    assert_eq!(decoded_config.name, "bincode");
    assert!(decoded_config.inline_config.is_map());
    assert!(decoded_config.linked_config.is_link());
}
