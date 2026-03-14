use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub brain: BrainConfig,
    pub bitget: BitgetConfig,
    pub execution: ExecutionConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Deserialize)]
pub struct BrainConfig {
    pub url: String,
    pub api_key: String,
    pub poll_interval_ms: u64,
}

#[derive(Debug, Deserialize)]
pub struct BitgetConfig {
    pub api_key: String,
    pub secret_key: String,
    pub passphrase: String,
    pub base_url: String,
}

#[derive(Debug, Deserialize)]
pub struct ExecutionConfig {
    pub max_retries: u32,
    pub retry_delay_ms: u64,
    pub max_slippage_pct: f64,
    pub dry_run: bool,
}

#[derive(Debug, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub json: bool,
}

impl Settings {
    pub fn load() -> Result<Self, config::ConfigError> {
        let cfg = config::Config::builder()
            .add_source(config::File::with_name("config/default"))
            .add_source(config::File::with_name("config/local").required(false))
            .add_source(config::Environment::with_prefix("EXEC").separator("__"))
            .build()?;

        cfg.try_deserialize()
    }
}
