use crate::bitget::{BitgetClient, PlaceOrderRequest};
use crate::brain::{BrainClient, TradeResult, TradeSignal};
use crate::config_model::ExecutionConfig;
use crate::error::ExecError;
use std::time::Duration;
use tokio::time::sleep;

pub struct Executor {
    brain: BrainClient,
    bitget: BitgetClient,
    poll_interval: Duration,
    max_retries: u32,
    retry_delay: Duration,
    dry_run: bool,
}

impl Executor {
    pub fn new(brain: BrainClient, bitget: BitgetClient, cfg: &ExecutionConfig) -> Self {
        Self {
            brain,
            bitget,
            poll_interval: Duration::from_millis(500),
            max_retries: cfg.max_retries,
            retry_delay: Duration::from_millis(cfg.retry_delay_ms),
            dry_run: cfg.dry_run,
        }
    }

    pub async fn run(&mut self) -> Result<(), ExecError> {
        tracing::info!("Executor loop started — polling brain for signals");

        loop {
            match self.brain.poll_signals().await {
                Ok(signals) => {
                    for signal in signals {
                        tracing::info!(
                            id = %signal.id,
                            symbol = %signal.symbol,
                            side = %signal.side,
                            qty = %signal.quantity,
                            "Received signal"
                        );
                        self.execute_signal(&signal).await;
                    }
                }
                Err(e) => {
                    tracing::warn!(error = %e, "Failed to poll brain — will retry");
                }
            }

            sleep(self.poll_interval).await;
        }
    }

    async fn execute_signal(&self, signal: &TradeSignal) {
        let result = if self.dry_run {
            tracing::info!(id = %signal.id, "DRY-RUN: would execute order");
            TradeResult {
                signal_id: signal.id.clone(),
                status: "dry_run".into(),
                exchange_order_id: None,
                filled_price: signal.price.clone(),
                filled_qty: Some(signal.quantity.clone()),
                error: None,
            }
        } else {
            self.place_with_retries(signal).await
        };

        if let Err(e) = self.brain.report_result(&result).await {
            tracing::error!(
                signal_id = %signal.id,
                error = %e,
                "Failed to report result to brain"
            );
        }
    }

    async fn place_with_retries(&self, signal: &TradeSignal) -> TradeResult {
        let req = PlaceOrderRequest {
            symbol: signal.symbol.clone(),
            side: signal.side.clone(),
            order_type: signal.order_type.clone(),
            size: signal.quantity.clone(),
            price: signal.price.clone(),
        };

        let mut last_error = String::new();

        for attempt in 1..=self.max_retries {
            match self.bitget.place_order(&req).await {
                Ok(order_id) => {
                    tracing::info!(
                        signal_id = %signal.id,
                        order_id = %order_id,
                        attempt,
                        "Order placed successfully"
                    );
                    return TradeResult {
                        signal_id: signal.id.clone(),
                        status: "filled".into(),
                        exchange_order_id: Some(order_id),
                        filled_price: signal.price.clone(),
                        filled_qty: Some(signal.quantity.clone()),
                        error: None,
                    };
                }
                Err(e) => {
                    last_error = e.to_string();
                    tracing::warn!(
                        signal_id = %signal.id,
                        attempt,
                        max = self.max_retries,
                        error = %last_error,
                        "Order attempt failed"
                    );
                    if attempt < self.max_retries {
                        sleep(self.retry_delay).await;
                    }
                }
            }
        }

        TradeResult {
            signal_id: signal.id.clone(),
            status: "error".into(),
            exchange_order_id: None,
            filled_price: None,
            filled_qty: None,
            error: Some(last_error),
        }
    }
}
