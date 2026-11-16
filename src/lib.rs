//! BIT Agents: one Aomi agent that runs every strategy built on BITAGENTS.
//!
//! A strategy is a spec stored by the BITAGENTS strategy service. Each
//! tool call names the strategy by id; the service decides and returns orders for
//! the connected Solana wallet. This app never signs or sends anything.

mod tool;

pub use tool::{Run, Status};

use aomi_sdk::{Secret, dyn_aomi_app};

/// Bump on every change so a deployed build can be told apart.
pub const BUILD: &str = "v4";

/// The BITAGENTS strategy service. Override with the BITAGENTS_API_URL secret.
pub const DEFAULT_API_URL: &str = "https://bitagents.aomi.dev";

#[derive(Clone, Default)]
pub struct BitAgents;

dyn_aomi_app!(
    app = BitAgents,
    name = "bitagents",
    version = "0.1.0",
    preamble = "You are the BIT Agents agent. You run trading strategies that users built on BITAGENTS. \
                Call them strategies, never agents: you are the agent. Each strategy has an id such as \
                ag1ea6b8de; if the user has not given one, ask for it. bitagents_status reads a strategy's \
                rules, limits, signals and its paper balance or the wallet's holdings; use it to answer how \
                a strategy is doing, giving equity and profit or loss against the starting balance. \
                bitagents_run decides and returns orders. A strategy's limits are enforced by the strategy \
                service; you cannot raise them. Follow the run tool's `next` field. In live mode: stage \
                each order with svm_stage_ix exactly as returned, simulate, then commit. In paper mode: \
                the orders were already simulated on mainnet, so report the fills and stage nothing. \
                Never build a swap yourself, never trade a token the run tool did not return, and never \
                state a price or signal you did not get from a tool.",
    tools = [Run, Status],
    secrets = [
        Secret::new("BITAGENTS_APP_KEY", "Key this app uses to call the BITAGENTS strategy service.", false),
        Secret::new("BITAGENTS_API_URL", "Base URL of the BITAGENTS strategy service.", false),
    ],
    namespaces = ["aomi-core", "svm-reads", "svm-write-ix"],
    skills = [{
        id: "bitagents/strategy",
        description: "How BITAGENTS strategies decide, what paper and live mode mean, and how to report a run",
        tags: ["strategy", "trading"],
        sections: { instructions: "skill/strategy.md" },
    }],
);
