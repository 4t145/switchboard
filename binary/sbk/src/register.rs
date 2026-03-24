use std::{ffi::OsStr, path::PathBuf};

use switchboard_http::HttpProvider;
use switchboard_kernel::KernelContext;
use switchboard_pf::PortForwardProvider;
use switchboard_socks5::Socks5Provider;
use switchboard_tcp::PortForwardProvider as TcpProvider;
use switchboard_uds::UdsProvider;

#[cfg(target_os = "linux")]
const LIB_EXT: &str = "so";
#[cfg(target_os = "macos")]
const LIB_EXT: &str = "dynlib";
#[cfg(target_os = "windows")]
const LIB_EXT: &str = "dll";
pub async fn register_prelude(context: &KernelContext) {
    context.register_service(Socks5Provider).await;
    context.register_service(PortForwardProvider).await;
    // http loading
    {
        let libs = &context.kernel_config.provider.http.plugins;
        let mut file_collection = vec![];
        let mut rust_dyn_libs = vec![];
        for lib in libs {
            if let Some(dir) = lib.strip_suffix("/*") {
                let Ok(mut dir) = tokio::fs::read_dir(dir)
                    .await
                    .inspect_err(|e| tracing::error!("fail to read http plugin dir {dir:?}: {e}"))
                else {
                    continue;
                };
                while let Ok(Some(file)) = dir.next_entry().await {
                    let file_path = file.path();
                    if file_path.is_file() && file.path().extension() == Some(OsStr::new(LIB_EXT)) {
                        tracing::debug!("collect http plugin lib {file_path:?}");
                        file_collection.push(file_path);
                    }
                }
            } else {
                let lib = PathBuf::from(lib);
                if lib.is_file() && lib.extension() == Some(OsStr::new(LIB_EXT)) {
                    tracing::debug!("collect http plugin lib {lib:?}");
                    file_collection.push(lib.clone());
                }
            }
        }
        unsafe {
            for lib_path in file_collection {
                if let Ok(lib) = libloading::Library::new(&lib_path).inspect_err(|e| {
                    tracing::error!("fail to load dynamic lib http plugin {lib_path:?}: {e}")
                }) {
                    rust_dyn_libs.push(lib);
                }
            }
        }
        context
            .register_service(HttpProvider { rust_dyn_libs })
            .await;
    }
    context.register_service(UdsProvider).await;
    context.register_service(TcpProvider).await;
}
