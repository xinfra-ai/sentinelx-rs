//! SentinelX Rust SDK
//! Pre-execution enforcement at the commit boundary.
//! <https://sentinelx.ai>

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

const DEFAULT_BASE_URL: &str = "https://enforce.sentinelx.ai";
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// A single invariant violation.
#[derive(Debug, Clone, Deserialize)]
pub struct Violation {
    pub primitive: String,
    pub code: String,
    pub constraint: String,
    pub message: String,
}

/// The enforcement receipt returned on every decision.
#[derive(Debug, Clone, Deserialize)]
pub struct Receipt {
    pub verdict: String,
    pub summary: String,
    pub constraint: Option<String>,
    pub constraint_pack: String,
    pub violation_code: Option<String>,
    pub violations: Vec<Violation>,
    pub mode: String,
    pub envelope_class: String,
    pub trace_id: String,
    pub request_hash: String,
    pub receipt_hash: String,
    pub inv_version: String,
    pub latency_ms: u64,
}

/// Error type for SentinelX operations.
#[derive(Debug, Error)]
pub enum SentinelXError {
    #[error("INADMISSIBLE: {0}")]
    Inadmissible(String),
    #[error("HTTP error: {0}")]
    Http(String),
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
}

impl SentinelXError {
    /// Returns the receipt if this is an AdmissibilityError.
    pub fn receipt(&self) -> Option<&Receipt> {
        None // receipt stored separately via inadmissible_receipt()
    }
}

/// Wrapper for INADMISSIBLE results — contains the full receipt.
#[derive(Debug)]
pub struct AdmissibilityError {
    pub receipt: Receipt,
}

impl AdmissibilityError {
    pub fn receipt(&self) -> &Receipt {
        &self.receipt
    }
}

impl std::fmt::Display for AdmissibilityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "INADMISSIBLE: {}", self.receipt.summary)
    }
}

impl std::error::Error for AdmissibilityError {}

#[derive(Serialize)]
struct EnforceRequest<'a> {
    action: &'a str,
    context: &'a HashMap<String, serde_json::Value>,
}

/// SentinelX enforcement client.
pub struct SentinelX {
    api_key: String,
    base_url: String,
}

impl SentinelX {
    /// Create a new client with the given API key.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: DEFAULT_BASE_URL.to_string(),
        }
    }

    /// Create a new client with a custom base URL.
    pub fn with_base_url(api_key: impl Into<String>, base_url: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: base_url.into(),
        }
    }

    /// Evaluate action admissibility at the commit boundary.
    /// Returns Receipt on ADMISSIBLE. Returns Err with AdmissibilityError on INADMISSIBLE.
    pub fn enforce(
        &self,
        action: &str,
        context: &HashMap<String, serde_json::Value>,
    ) -> Result<Receipt, Box<dyn std::error::Error>> {
        let url = format!("{}/v2/enforce", self.base_url);
        let body = EnforceRequest { action, context };

        let resp = ureq::post(&url)
            .set("Content-Type", "application/json")
            .set("X-API-Key", &self.api_key)
            .set("User-Agent", &format!("sentinelx-rs/{}", VERSION))
            .send_json(serde_json::to_value(&body)?);

        match resp {
            Ok(r) => {
                let receipt: Receipt = r.into_json()?;
                Ok(receipt)
            }
            Err(ureq::Error::Status(_, r)) => {
                let receipt: Receipt = r.into_json()?;
                Err(Box::new(AdmissibilityError { receipt }))
            }
            Err(e) => Err(Box::new(SentinelXError::Http(e.to_string()))),
        }
    }

    /// Always returns the receipt. Never errors on INADMISSIBLE.
    pub fn evaluate(
        &self,
        action: &str,
        context: &HashMap<String, serde_json::Value>,
    ) -> Result<Receipt, Box<dyn std::error::Error>> {
        match self.enforce(action, context) {
            Ok(r) => Ok(r),
            Err(e) => {
                if let Some(ae) = e.downcast_ref::<AdmissibilityError>() {
                    Ok(ae.receipt.clone())
                } else {
                    Err(e)
                }
            }
        }
    }
}
