use schemars::JsonSchema;
use std::fs::File;
use switchboard_http::router::{HostRouterConfig, PathMatchRouterConfig};
macro_rules! export_types {
    ($generator: ident $T: ty) => {{
        let dir = dir();
        let mut file = File::create(format!("{dir}/{}.schema.json", stringify!($T)))?;
        let schema = $generator.root_schema_for::<$T>();
        serde_json::to_writer_pretty(&mut file, &schema)?;
    }};
    ($generator: ident $($T: ty)*) => {{
        $(
            export_types!($generator $T);
        )*
    }};
}
fn dir() -> String {
    std::env::var("OUT_DIR").unwrap_or("utils/sbh-config/schema".into())
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = dir();
    std::fs::create_dir_all(&dir)?;
    let mut config = schemars::generate::SchemaSettings::draft07();
    config.inline_subschemas = true;
    let mut generator = config.into_generator();
    export_types!(generator
        PathMatchRouterConfig
        HostRouterConfig
    );
    Ok(())
}
