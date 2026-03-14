use crate::config_model::BitgetConfig;
use crate::error::ExecError;
use hmac::{Hmac, Mac};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaceOrderRequest {
    pub symbol: String,
    pub side: String,
    pub order_type: String,
    pub size: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BitgetResponse<T> {
    pub code: String,
    pub msg: String,
    pub data: Option<T>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderData {
    pub order_id: String,
}

pub struct BitgetClient {
    client: Client,
    base_url: String,
    api_key: String,
    secret_key: String,
    passphrase: String,
}

impl BitgetClient {
    pub fn new(cfg: &BitgetConfig) -> Self {
        Self {
            client: Client::new(),
            base_url: cfg.base_url.clone(),
            api_key: cfg.api_key.clone(),
            secret_key: cfg.secret_key.clone(),
            passphrase: cfg.passphrase.clone(),
        }
    }

    fn sign(&self, timestamp: &str, method: &str, path: &str, body: &str) -> String {
        let message = format!("{timestamp}{method}{path}{body}");
        let mut mac =
            HmacSha256::new_from_slice(self.secret_key.as_bytes()).expect("HMAC key error");
        mac.update(message.as_bytes());
        use base64::Engine;
        base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes())
    }

    pub async fn place_order(
        &self,
        req: &PlaceOrderRequest,
    ) -> Result<String, ExecError> {
        let path = "/api/v2/spot/trade/place-order";
        let timestamp = chrono::Utc::now().timestamp_millis().to_string();
        let body = serde_json::to_string(req).unwrap();
        let signature = self.sign(&timestamp, "POST", path, &body);

        let resp = self
            .client
            .post(format!("{}{}", self.base_url, path))
            .header("ACCESS-KEY", &self.api_key)
            .header("ACCESS-SIGN", &signature)
            .header("ACCESS-TIMESTAMP", &timestamp)
            .header("ACCESS-PASSPHRASE", &self.passphrase)
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await?;

        let status = resp.status();
        let text = resp.text().await?;

        let parsed: BitgetResponse<OrderData> =
            serde_json::from_str(&text).map_err(|e| ExecError::Bitget(format!("{e}: {text}")))?;

        if parsed.code != "00000" {
            return Err(ExecError::Bitget(format!(
                "HTTP {status} — code={}, msg={}",
                parsed.code, parsed.msg
            )));
        }

        let order_id = parsed
            .data
            .map(|d| d.order_id)
            .unwrap_or_else(|| "unknown".into());

        Ok(order_id)
    }
}
