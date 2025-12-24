use switchboard_http::HttpProvider;
use switchboard_kernel::KernelContext;
use switchboard_pf::PortForwardProvider;
use switchboard_socks5::Socks5Provider;
use switchboard_uds::UdsProvider;

pub async fn register_prelude(context: &KernelContext) {
    context.register_service(Socks5Provider).await;
    context.register_service(PortForwardProvider).await;
    context.register_service(HttpProvider).await;
    context.register_service(UdsProvider).await;
}
