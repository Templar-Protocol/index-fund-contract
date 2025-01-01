use near_sdk::json_types::U64;
use serde_json::json;

#[tokio::test]
async fn test_contract_is_operational() -> Result<(), Box<dyn std::error::Error>> {
    let sandbox = near_workspaces::sandbox().await?;
    let contract_wasm = near_workspaces::compile_project("./").await?;

    // Deploy contract
    let contract = sandbox.dev_deploy(&contract_wasm).await?;

    // Initialize contract with 1 day rebalance interval (86400 seconds)
    let outcome = contract
        .call("new")
        .args_json(json!({
            "rebalance_interval": U64::from(86400)
        }))
        .transact()
        .await?;
    println!("Outcome: {:?}", outcome);
    assert!(outcome.is_success());

    // Create curator account
    let curator = sandbox.dev_create_account().await?;

    // Register curator (with attached deposit for storage)
    let outcome = curator
        .call(contract.id(), "register_curator")
        .args_json(json!({
            "curator_address": curator.id()
        }))
        .deposit(near_sdk::NearToken::from_near(1)) // 1 NEAR should be enough for storage
        .transact()
        .await?;
    assert!(outcome.is_success());

    // Set initial weights
    let outcome = curator
        .call(contract.id(), "update_weights")
        .args_json(json!({
            "updates": [
                {
                    "weight": U64::from(6000),
                    "asset_address": "asset1.near"
                },
                {
                    "weight": U64::from(4000),
                    "asset_address": "asset2.near"
                }
            ]
        }))
        .transact()
        .await?;
    assert!(outcome.is_success());

    // Verify weights were set correctly
    let weights = contract.view("get_weights").args_json(json!({})).await?;
    let weights: Vec<serde_json::Value> = weights.json()?;
    assert_eq!(weights.len(), 2);

    Ok(())
}
