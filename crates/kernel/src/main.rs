use std::vec;

use serde_json::json;
use switchboard_http::{
    object::{
        Object, ObjectId,
        class::{ObjectClassName, RouterProperty, ServiceProperty},
        registry::ObjectRegistry,
    },
    router::Route,
};
use switchboard_kernel::{KernelContext, config::mem::Mem};
use switchboard_model::{AnonServiceDescriptor, Bind, NamedService, ServiceDescriptor};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let mut config = Mem::new();
    config.add_named_service(
        NamedService::builder()
            .name("mc-pf")
            .config("192.168.1.5:25565")
            .description("forward minecraft traffic")
            .provider("pf")
            .build(),
    );
    config.add_bind(
        Bind::builder()
            .addr("[::]:25565".parse()?)
            .description("mc")
            .service(ServiceDescriptor::named("mc-pf"))
            .build(),
    );
    config.add_bind(
        Bind::builder()
            .addr("[::]:10999".parse()?)
            .description("socks5 proxy")
            .service(AnonServiceDescriptor::builder().provider("socks5").build())
            .build(),
    );

    let mut http_objects = ObjectRegistry::new();
    let client_object_id = ObjectId::new("client");
    let rewrite_object_id = ObjectId::new("rewrite");
    let router_object_id = ObjectId::new("router");
    http_objects.service.insert(
        client_object_id.clone(),
        Object {
            id: client_object_id.clone(),
            class: ObjectClassName::std("client"),
            config: "".to_string(),
            property: ServiceProperty {
                layers: vec![rewrite_object_id.clone()],
            },
        },
    );
    http_objects.router.insert(
        router_object_id.clone(),
        Object {
            id: router_object_id.clone(),
            class: ObjectClassName::std("path-match"),
            config: json!(
                [
                    {
                        "path": "/baidu/{*path}",
                        "template": "/{path}",
                    },
                    {
                        "path": "/{*path}",
                        "template": "/{{path}}",
                        "priority:": -1
                    },
                ]
            )
            .to_string(),
            property: RouterProperty {
                routes: [(Route::Fallback, client_object_id.clone())].into(),
                layers: vec![],
            },
        },
    );
    http_objects.layer.insert(
        rewrite_object_id.clone(),
        Object {
            id: rewrite_object_id,
            class: ObjectClassName::std("rewrite"),
            config: json!(
                {
                    "host": "baidu.com",
                    "schema": "http",
                }
            )
            .to_string(),
            property: (),
        },
    );
    // http_objects.
    config.add_named_service(
        NamedService::builder()
            .name("http-gateway")
            .config(
                json!({
                    "objects": http_objects,
                    "entrypoint": router_object_id
                })
                .to_string(),
            )
            .description("http gateway")
            .provider("http")
            .build(),
    );
    config.add_bind(
        Bind::builder()
            .addr("[::]:2525".parse()?)
            .service(ServiceDescriptor::named("http-gateway"))
            .build(),
    );
    let mut context = KernelContext::startup(config).await?;
    tracing::info!("Kernel startup complete");
    tracing::info!("Kernel running, press Ctrl+C to exit");
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install Ctrl+C signal handler");
    tracing::info!("Ctrl+C signal received, shutting down...");
    context.supervisor.shutdown().await;
    Ok(())
}
