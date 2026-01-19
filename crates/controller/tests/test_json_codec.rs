use chrono::Utc;
use switchboard_controller::{
    link_resolver::Link,
    storage::{
        JsonInterpreter, KnownObject, StorageMeta, StorageObjectDescriptor, StorageObjectValueStyle,
    },
};
use switchboard_model::{
    HumanReadableServiceConfig, Listener, ServiceConfig, TcpServiceConfig,
    switchboard_serde_value::{SerdeValue, value},
    tcp_route,
};

#[test]
fn test_human_readable_service_config_conversion() {
    let json_raw: SerdeValue = serde_json::from_str(include_str!(
        "test_json_codec/human_readable_service_config.json"
    ))
    .unwrap();
    println!(
        "Loaded SerdeValue from JSON:\n{:#?}",
        json_raw
    );
    let human_readable_config: HumanReadableServiceConfig<Link> =
        json_raw.deserialize_into().unwrap();
    let json = serde_json::to_string_pretty(&human_readable_config).unwrap();
    println!("HumanReadableServiceConfig<Link>:\n{}", json);
}
// #[test]
// fn test_json_codec() {
//     let tcp_service_1 = TcpServiceConfig {
//         provider: "example-provider".to_string(),
//         name: "example-service".to_string(),
//         config: Some(value!({
//             "host": "127.0.0.1",
//         })),
//         description: None,
//     };
//     let bind_ip = "0.0.0.0:9080".parse().unwrap();
//     let tcp_listener_1 = Listener {
//         bind: bind_ip,
//         description: None,
//     };
//     let tcp_route_1 = tcp_route::TcpRoute {
//         bind: bind_ip,
//         service: "example-service".to_string(),
//         tls: None,
//     };
//     let example_service_config: ServiceConfig = ServiceConfig {
//         tcp_services: vec![("example-service".to_string(), tcp_service_1)]
//             .into_iter()
//             .collect(),
//         tcp_listeners: vec![(bind_ip, tcp_listener_1)].into_iter().collect(),
//         tcp_routes: vec![(bind_ip, tcp_route_1)].into_iter().collect(),
//         tls: std::collections::BTreeMap::new(),
//     };
//     let data = SerdeValue::serialize_from(&example_service_config).unwrap();
//     println!("Serialized ServiceConfig to SerdeValue: {:#?}", data);
//     let obj = StorageObjectValueStyle {
//         descriptor: StorageObjectDescriptor {
//             id: "example-service-config".to_string(),
//             revision: "1".to_string(),
//         },
//         meta: StorageMeta {
//             data_type: ServiceConfig::data_type().to_string(),
//             created_at: Utc::now(),
//         },
//         data: SerdeValue::serialize_from(&example_service_config).unwrap(),
//     };

//     let json_value = JsonInterpreter::encode(obj).unwrap();
//     let decoded_obj = JsonInterpreter::decode(json_value, ServiceConfig::data_type()).unwrap();
//     let decoded_service_config: ServiceConfig = decoded_obj.deserialize_into().unwrap();
//     assert_eq!(example_service_config, decoded_service_config);
// }
