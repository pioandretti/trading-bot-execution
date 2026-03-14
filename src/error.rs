use thiserror::Error;

#[derive(Debug, Error)]
pub enum ExecError {
    #[error("Brain API error: {0}")]
    Brain(String),

    #[error("Bitget API error: {0}")]
    Bitget(String),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Slippage {actual_pct:.2}% exceeds max {max_pct:.2}%")]
    SlippageExceeded { actual_pct: f64, max_pct: f64 },

    #[error("Config error: {0}")]
    Config(#[from] config::ConfigError),
}
