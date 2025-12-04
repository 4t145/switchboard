
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
        tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();
    let context = switchboard_controller::ControllerContext::new(Default::default());
    context.startup().await?; 
    context.take_over_all_kernels().await?;
    tracing::info!("Controller started, press Ctrl+C to exit");
    tokio::signal::ctrl_c().await?;
    tracing::info!("Controller shutting down");
    context.shutdown().await?;
    Ok(())
}


