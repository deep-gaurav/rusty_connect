use std::{
    path::{Path},
};

use rusty_connect::{
    RustyConnect,
};

use tracing::info;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    let fmt_layer = fmt::layer().with_target(true);
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .init();

    info!("RUNNING");
    let device_name = "RustyConnect";
    let device_id = uuid::Uuid::new_v4().to_string();
    let device_type = "laptop";

    let mut rusty = RustyConnect::new(
        &device_id,
        device_name,
        device_type,
        Path::new("example_configs"),
    )
    .await?;

    rusty.run(3000).await?;
    Ok(())
}
