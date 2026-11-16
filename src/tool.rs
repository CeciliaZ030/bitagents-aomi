use std::time::Duration;

use aomi_sdk::{DynAomiTool, DynToolCallCtx, resolve_secret_value};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::{BUILD, BitAgents, DEFAULT_API_URL};

/// A strategy id is `ag` followed by lowercase hex, as issued by BITAGENTS.
fn strategy_id(raw: &str) -> Result<&str, String> {
    let id = raw.trim();
    let ok = id.len() >= 6
        && id.len() <= 40
        && id.starts_with("ag")
        && id[2..]
            .chars()
            .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c));
    if ok {
        Ok(id)
    } else {
        Err(format!(
            "`{raw}` is not a BITAGENTS strategy id (expected something like ag1ea6b8de)"
        ))
    }
}

fn api(ctx: &DynToolCallCtx, strategy: &str, body: Value) -> Result<Value, String> {
    let base = resolve_secret_value(ctx, None, "BITAGENTS_API_URL", "")
        .unwrap_or_else(|_| DEFAULT_API_URL.to_string());
    // Sent only when configured; the service enforces it only when it has one set.
    let key = resolve_secret_value(ctx, None, "BITAGENTS_APP_KEY", "").ok();
    let url = format!(
        "{}/builder/agents/{strategy}/evaluate",
        base.trim_end_matches('/')
    );
    let resp = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(90))
        .build()
        .map_err(|e| format!("http client: {e}"))?
        .post(&url)
        .json(&body);
    let resp = match key {
        Some(key) => resp.header("x-app-key", key),
        None => resp,
    };
    let resp = resp
        .send()
        .map_err(|e| format!("BITAGENTS strategy service unreachable: {e}"))?;
    let status = resp.status();
    let value: Value = resp
        .json()
        .map_err(|e| format!("bad response from strategy service: {e}"))?;
    if !status.is_success() {
        let detail = value
            .get("detail")
            .and_then(Value::as_str)
            .unwrap_or("request failed");
        return Err(format!("strategy service returned {status}: {detail}"));
    }
    Ok(value)
}

fn wallet(ctx: &DynToolCallCtx) -> Result<String, String> {
    ctx.attribute_string(&["domain", "svm", "address"])
        .filter(|a| !a.trim().is_empty())
        .ok_or_else(|| "no Solana wallet is connected to this session".to_string())
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct StrategyArgs {
    /// The BITAGENTS strategy id, for example ag1ea6b8de.
    #[serde(alias = "agent_id")]
    pub strategy_id: String,
}

/// The service still says "agent" for what users call a strategy; rename before the model sees it.
fn relabel(mut out: Value) -> Value {
    if let Some(map) = out.as_object_mut() {
        if let Some(template) = map.remove("strategy") {
            map.insert("template".into(), template);
        }
        if let Some(name) = map.remove("agent") {
            map.insert("strategy".into(), name);
        }
        if let Some(id) = map.remove("agent_id") {
            map.insert("strategy_id".into(), id);
        }
    }
    out
}

pub struct Status;

impl DynAomiTool for Status {
    type App = BitAgents;
    type Args = StrategyArgs;

    const NAME: &'static str = "bitagents_status";
    const DESCRIPTION: &'static str = "Show a BITAGENTS strategy's rules, limits, paper balance, the connected wallet's holdings and the latest signals. Read-only: never trades.";

    fn run(_app: &BitAgents, args: StrategyArgs, ctx: DynToolCallCtx) -> Result<Value, String> {
        let strategy = strategy_id(&args.strategy_id)?;
        let wallet = wallet(&ctx)?;
        let mut out = relabel(api(
            &ctx,
            strategy,
            json!({ "wallet": wallet, "build": false }),
        )?);
        out["app_build"] = json!(BUILD);
        Ok(out)
    }
}

pub struct Run;

impl DynAomiTool for Run {
    type App = BitAgents;
    type Args = StrategyArgs;

    const NAME: &'static str = "bitagents_run";
    const DESCRIPTION: &'static str = "Run a BITAGENTS strategy once for the connected wallet. Returns the decision and a `next` field saying what to do. A live strategy returns exact unsigned swap instructions to stage with svm_stage_ix; a paper strategy returns fills already simulated on mainnet. Does not sign or send anything itself.";

    fn run(_app: &BitAgents, args: StrategyArgs, ctx: DynToolCallCtx) -> Result<Value, String> {
        let strategy = strategy_id(&args.strategy_id)?;
        let wallet = wallet(&ctx)?;
        let mut out = relabel(api(
            &ctx,
            strategy,
            json!({ "wallet": wallet, "build": true }),
        )?);
        if out.get("next").is_none() {
            out["next"] = json!("Report the signals and notes to the user. Do not stage anything.");
        }
        out["app_build"] = json!(BUILD);
        Ok(out)
    }
}
