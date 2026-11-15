use aomi_sdk::testing::{TestCtxBuilder, run_tool};
use aomi_sdk::{DynAomiApp, DynAomiTool, validate_app_skills_with_tools};
use bitagents::{BitAgents, Run, Status};
use serde_json::{Value, json};

const WALLET: &str = "11111111111111111111111111111112";

#[test]
fn skills_and_tools_are_valid() {
    let manifest = BitAgents.manifest();
    validate_app_skills_with_tools(&manifest.name, &manifest.skills, &manifest.tools)
        .expect("app manifest is valid");
}

/// The model provider rejects the whole tool list if any object schema lacks `properties`.
#[test]
fn every_object_schema_has_properties() {
    fn check(node: &Value, path: &str) {
        match node {
            Value::Object(map) => {
                if map.get("type") == Some(&json!("object")) {
                    assert!(
                        map.get("properties").is_some_and(Value::is_object),
                        "{path}: object schema without properties"
                    );
                }
                for (k, v) in map {
                    check(v, &format!("{path}/{k}"));
                }
            }
            Value::Array(items) => {
                for (i, v) in items.iter().enumerate() {
                    check(v, &format!("{path}/{i}"));
                }
            }
            _ => {}
        }
    }
    let manifest = serde_json::to_value(BitAgents.manifest()).unwrap();
    check(&manifest, "manifest");
}

#[test]
fn rejects_a_malformed_agent_id() {
    let ctx = TestCtxBuilder::new(Run::NAME).build();
    let err = run_tool::<Run>(&BitAgents, json!({ "agent_id": "../admin" }), ctx)
        .err()
        .unwrap();
    assert!(err.contains("not a BITAGENTS agent id"), "{err}");
}

#[test]
fn run_needs_a_connected_solana_wallet() {
    let ctx = TestCtxBuilder::new(Run::NAME).build();
    let err = run_tool::<Run>(&BitAgents, json!({ "agent_id": "ag1ea6b8de" }), ctx)
        .err()
        .unwrap();
    assert!(err.contains("no Solana wallet"), "{err}");
}

#[test]
fn status_needs_the_service_configured() {
    let ctx = TestCtxBuilder::new(Status::NAME)
        .attribute("domain", json!({ "svm": { "address": WALLET } }))
        .build();
    let err = run_tool::<Status>(&BitAgents, json!({ "agent_id": "ag1ea6b8de" }), ctx)
        .err()
        .unwrap();
    assert!(err.contains("BITAGENTS_API_URL"), "{err}");
}
