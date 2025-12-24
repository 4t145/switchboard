use switchboard_custom_config::LinkOrValue;

#[test]
fn test_deserialize_link_or_string() {
    let json_str = r#""file:///path/to/config.json""#;
    let link: LinkOrValue<String> = serde_json::from_str(json_str).unwrap();
    match link {
        LinkOrValue::Link(link) => {
            assert!(link.is_file());
            assert_eq!(link.to_string(), "file:///path/to/config.json");
        }
        _ => panic!("Expected Link variant"),
    }
    let json_str = r#""some plain string value""#;
    let value: LinkOrValue<String> = serde_json::from_str(json_str).unwrap();
    match value {
        LinkOrValue::Value(s) => {
            assert_eq!(s, "some plain string value");
        }
        _ => panic!("Expected Value variant"),
    }
}