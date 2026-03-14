mod config_model;
mod error;
mod bitget;
mod brain;
mod executor;

use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = config_model::Settings::load()?;

    // Initialise tracing
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&settings.logging.level));

    if settings.logging.json {
        fmt().json().with_env_filter(filter).init();
    } else {
        fmt().with_env_filter(filter).init();
    }

    tracing::info!(
        dry_run = settings.execution.dry_run,
        "Trading Execution Bot starting"
    );

    if settings.execution.dry_run {
        tracing::warn!("DRY-RUN mode — no real orders will be placed");
    }

    let brain_client = brain::BrainClient::new(&settings.brain);
    let bitget_client = bitget::BitgetClient::new(&settings.bitget);
    let mut exec = executor::Executor::new(brain_client, bitget_client, &settings.execution);

    exec.run().await?;

    Ok(())
}
