use std::time::Duration;

use aomi_sdk::{DynAomiTool, DynToolCallCtx, resolve_secret_value};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::{BUILD, BitAgents, DEFAULT_API_URL};

/// An agent id is `ag` followed by lowercase hex, as issued by BITAGENTS.
fn agent_id(raw: &str) -> Result<&str, String> {
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
            "`{raw}` is not a BITAGENTS agent id (expected something like ag1ea6b8de)"
        ))
    }
}

fn api(ctx: &DynToolCallCtx, agent: &str, body: Value) -> Result<Value, String> {
    let base = resolve_secret_value(ctx, None, "BITAGENTS_API_URL", "")
        .unwrap_or_else(|_| DEFAULT_API_URL.to_string());
    // Sent only when configured; the service enforces it only when it has one set.
    let key = resolve_secret_value(ctx, None, "BITAGENTS_APP_KEY", "").ok();
    let url = format!(
        "{}/builder/agents/{agent}/evaluate",
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
pub struct AgentArgs {
    /// The BITAGENTS agent id, for example ag1ea6b8de.
    pub agent_id: String,
}

pub struct Status;

impl DynAomiTool for Status {
    type App = BitAgents;
    type Args = AgentArgs;

    const NAME: &'static str = "bitagents_status";
    const DESCRIPTION: &'static str = "Show a BITAGENTS agent's strategy, limits, the connected wallet's holdings and the latest signals. Read-only: never trades.";

    fn run(_app: &BitAgents, args: AgentArgs, ctx: DynToolCallCtx) -> Result<Value, String> {
        let agent = agent_id(&args.agent_id)?;
        let wallet = wallet(&ctx)?;
        let mut out = api(&ctx, agent, json!({ "wallet": wallet, "build": false }))?;
        out["app_build"] = json!(BUILD);
        Ok(out)
    }
}

pub struct Run;

impl DynAomiTool for Run {
    type App = BitAgents;
    type Args = AgentArgs;

    const NAME: &'static str = "bitagents_run";
    const DESCRIPTION: &'static str = "Run a BITAGENTS agent's strategy once for the connected wallet. Returns the decision and a `next` field saying what to do. A live agent returns exact unsigned swap instructions to stage with svm_stage_ix; a paper agent returns fills already simulated on mainnet. Does not sign or send anything itself.";

    fn run(_app: &BitAgents, args: AgentArgs, ctx: DynToolCallCtx) -> Result<Value, String> {
        let agent = agent_id(&args.agent_id)?;
        let wallet = wallet(&ctx)?;
        let mut out = api(&ctx, agent, json!({ "wallet": wallet, "build": true }))?;
        if out.get("next").is_none() {
            out["next"] = json!("Report the signals and notes to the user. Do not stage anything.");
        }
        out["app_build"] = json!(BUILD);
        Ok(out)
    }
}
