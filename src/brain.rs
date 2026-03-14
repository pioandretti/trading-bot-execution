use crate::config_model::BrainConfig;
use crate::error::ExecError;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeSignal {
    pub id: String,
    pub symbol: String,
    pub side: String,           // "buy" | "sell"
    pub order_type: String,     // "market" | "limit"
    pub quantity: String,
    pub price: Option<String>,
    pub stop_loss: Option<String>,
    pub take_profit: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TradeResult {
    pub signal_id: String,
    pub status: String,
    pub exchange_order_id: Option<String>,
    pub filled_price: Option<String>,
    pub filled_qty: Option<String>,
    pub error: Option<String>,
}

pub struct BrainClient {
    client: Client,
    base_url: String,
    api_key: String,
}

impl BrainClient {
    pub fn new(cfg: &BrainConfig) -> Self {
        Self {
            client: Client::new(),
            base_url: cfg.url.clone(),
            api_key: cfg.api_key.clone(),
        }
    }

    /// Poll the brain for pending trade signals.
    pub async fn poll_signals(&self) -> Result<Vec<TradeSignal>, ExecError> {
        let resp = self
            .client
            .get(format!("{}/api/signals/pending", self.base_url))
            .header("X-API-Key", &self.api_key)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(ExecError::Brain(format!("{status}: {body}")));
        }

        let signals: Vec<TradeSignal> = resp.json().await?;
        Ok(signals)
    }

    /// Report execution result back to the brain.
    pub async fn report_result(&self, result: &TradeResult) -> Result<(), ExecError> {
        let resp = self
            .client
            .post(format!("{}/api/signals/result", self.base_url))
            .header("X-API-Key", &self.api_key)
            .json(result)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(ExecError::Brain(format!("{status}: {body}")));
        }

        Ok(())
    }
}
