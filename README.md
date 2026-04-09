# sentinelx

Whether it's an AI agent, a bank transfer, or a power grid — execution is permanent.

`sentinelx` enforces at the commit boundary. Before the action executes. Server-side. Unbypassable. Cryptographic receipt on every decision.

[![license](https://img.shields.io/badge/license-Apache--2.0-blue)](LICENSE)
[![crates.io](https://img.shields.io/crates/v/sentinelx)](https://crates.io/crates/sentinelx)

## Install

```toml
[dependencies]
sentinelx = "0.1"
```

Or:

```bash
cargo add sentinelx
```

## Quick Start

```rust
use sentinelx::SentinelX;
use serde_json::json;
use std::collections::HashMap;

fn main() {
    let sx = SentinelX::new("YOUR_API_KEY");

    let mut ctx = HashMap::new();
    ctx.insert("agent_id".to_string(), json!("agent-001"));
    ctx.insert("action_type".to_string(), json!("file.write"));
    ctx.insert("human_in_loop_required".to_string(), json!(true));
    ctx.insert("human_in_loop".to_string(), json!(false));
    ctx.insert("action_within_scope".to_string(), json!(true));
    ctx.insert("action_logged".to_string(), json!(true));

    match sx.enforce("ai.agent.action.execute", &ctx) {
        Ok(receipt) => {
            println!("✅ ADMISSIBLE");
            println!("Receipt hash: {}", receipt.receipt_hash);
        }
        Err(e) => {
            if let Some(receipt) = e.receipt() {
                println!("❌ INADMISSIBLE: {}", receipt.summary);
                println!("Constraint: {:?}", receipt.constraint);
                println!("Violations: {}", receipt.violations.len());
                println!("Receipt hash: {}", receipt.receipt_hash);
                // Action never executed. Receipt sealed.
            }
        }
    }
}
```

## SCADA Example

```rust
let mut ctx = HashMap::new();
ctx.insert("device_id".to_string(), json!("rtu-456"));
ctx.insert("parameter".to_string(), json!("voltage_setpoint"));
ctx.insert("operator_authorized".to_string(), json!(true));
ctx.insert("change_ticket_linked".to_string(), json!(true));
ctx.insert("change_logged".to_string(), json!(true));
ctx.insert("two_person_auth".to_string(), json!(true));
ctx.insert("rollback_procedure_defined".to_string(), json!(true));
ctx.insert("action_logged".to_string(), json!(true));

let receipt = sx.enforce("scada.setpoint.change", &ctx)?;
println!("ADMISSIBLE | Receipt: {}", receipt.receipt_hash);
```

## Observe Mode — Never Errors on INADMISSIBLE

```rust
// Always returns receipt. Never errors on INADMISSIBLE.
// Useful for logging pipelines and observe mode.
let receipt = sx.evaluate("wire.transfer.execute", &ctx)?;
println!("{}", receipt.verdict); // "ADMISSIBLE" or "INADMISSIBLE"
```

## How It Works

SentinelX sits at the commit boundary between your system and execution. Before any irreversible action fires, the enforcement engine evaluates it against invariant constraints and returns a deterministic verdict with a provenance receipt.

- **ADMISSIBLE** → receipt returned, action may proceed
- **INADMISSIBLE** → `SentinelXError::Inadmissible` returned, nothing executes, receipt sealed

The enforcement decision is made server-side. It cannot be bypassed client-side.

## Receipt Shape

```rust
pub struct Receipt {
    pub verdict: String,         // "ADMISSIBLE" or "INADMISSIBLE"
    pub summary: String,         // human-readable decision
    pub constraint: Option<String>,   // matched invariant name
    pub constraint_pack: String, // action evaluated
    pub violation_code: Option<String>, // e.g. "INV-046"
    pub violations: Vec<Violation>,    // all matched invariants
    pub trace_id: String,        // unique per evaluation
    pub request_hash: String,    // sha256 of the request
    pub receipt_hash: String,    // sha256 sealed receipt
    pub inv_version: String,     // invariant catalog date
    pub latency_ms: u64,         // enforcement latency
}
```

## Domain Coverage

| Domain | Example Actions |
|--------|----------------|
| AI/ML Agents | `ai.agent.action.execute`, `ml.model.deploy.production` |
| Financial | `wire.transfer.execute`, `algo.trade.execute` |
| OT/SCADA | `scada.setpoint.change`, `breaker.open.execute` |
| Grid/Energy | `load.transfer.execute`, `der.curtailment.execute.batch` |
| Cyber/RMM | `rmm.script.execute`, `rmm.privilege.escalate` |
| Healthcare | `medication.order.execute`, `patient.record.modify` |
| Mobility | `driver.payout.execute`, `surge.pricing.apply` |

## Get an API Key

```bash
curl -X POST https://enforce.sentinelx.ai/generate-key
```

Or visit [sentinelx.ai](https://sentinelx.ai).

## Links

- [sentinelx.ai](https://sentinelx.ai)
- [enforce.sentinelx.ai](https://enforce.sentinelx.ai)
- [@sentinelx/sdk on npm](https://npmjs.com/package/@sentinelx/sdk)
- [sentinelx-go on GitHub](https://github.com/xinfra-ai/sentinelx-go)
- [sentinelx-sdk on PyPI](https://pypi.org/project/sentinelx-sdk)

## License

Apache-2.0
