// Find all our documentation at https://docs.near.org
use near_sdk::collections::UnorderedMap;
use near_sdk::json_types::{U128, U64};
use near_sdk::{env, require, AccountId};
use near_sdk::{near, NearToken};

use std::str::FromStr;

pub type AssetId = AccountId;
pub type Price = U128;

#[derive(Debug)]
#[near(serializers = [json, borsh])]
pub struct AssetWeight {
    pub weight: U64, // Basis points (e.g., 5000 = 50%)
    pub asset_address: AssetId,
}

#[near(serializers = [json, borsh])]
pub struct AssetHolding {
    pub balance: U128,
    pub weight: U64,
    pub last_price: U128,
    pub last_updated: U64,
}

// Define the contract structure
#[near(contract_state)]
pub struct IndexFund {
    // a Curator is normally a DAO, but could be any account
    pub curator_address: Option<AccountId>,
    pub assets: UnorderedMap<AssetId, AssetHolding>,
    pub last_rebalance: U64,
    // TODO: (LP) add support for milliseconds in addition to blocks
    pub rebalance_interval: U64, // blocks
}

impl Default for IndexFund {
    fn default() -> Self {
        Self {
            curator_address: None,
            assets: UnorderedMap::new(b"a"),
            last_rebalance: U64(0),
            rebalance_interval: U64(86400), // ~1 day assuming 1 block per second
        }
    }
}

// Implement the contract structure
#[near]
impl IndexFund {
    #[init]
    pub fn new(rebalance_interval: U64) -> Self {
        require!(rebalance_interval > U64(0), "Invalid rebalance interval");
        Self {
            curator_address: None,
            assets: UnorderedMap::new(b"a"),
            last_rebalance: U64(0),
            rebalance_interval,
        }
    }

    #[payable]
    pub fn register_curator(&mut self, curator_address: AccountId) {
        require!(self.curator_address.is_none(), "curator already registered");
        require!(
            env::attached_deposit()
                >= NearToken::from_yoctonear(env::storage_byte_cost().as_yoctonear() * 100),
            "Insufficient storage deposit"
        );
        self.curator_address = Some(curator_address);
    }

    pub fn update_weights(&mut self, updates: Vec<AssetWeight>) {
        let curator = self
            .curator_address
            .as_ref()
            .expect("curator not registered");
        require!(env::predecessor_account_id() == *curator, "Unauthorized");

        // Create a temporary copy of current weights
        let mut new_weights: std::collections::HashMap<AccountId, U64> =
            self.assets.iter().map(|(k, v)| (k, v.weight)).collect();

        // Apply updates
        for update in updates.iter() {
            new_weights.insert(update.asset_address.clone(), update.weight);
        }

        // Verify total weight is 10000 (100%)
        let total_weight: u64 = new_weights.values().map(|&w| u64::from(w)).sum();
        require!(total_weight == 10000, "Final weights must sum to 100%");

        // Apply updates only after verification
        for update in &updates {
            if let Some(mut holding) = self.assets.get(&update.asset_address) {
                holding.weight = update.weight;
                self.assets.insert(&update.asset_address, &holding);
            } else {
                self.assets.insert(
                    &update.asset_address,
                    &AssetHolding {
                        balance: U128(0),
                        weight: update.weight,
                        last_price: U128(0),
                        last_updated: U64(env::block_timestamp()),
                    },
                );
            }
        }

        env::log_str(&format!("Updated weights: {:?}", updates));
    }

    pub fn get_weights(&self) -> Vec<AssetWeight> {
        self.assets
            .iter()
            .map(|(asset_address, h)| AssetWeight {
                weight: h.weight,
                asset_address,
            })
            .collect()
    }

    pub fn get_assets(&self) -> Vec<AssetId> {
        self.assets.keys().collect()
    }
}

/*
 * =====================================================================
 * --------------------------------Tests--------------------------------
 * =====================================================================
 */
#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::testing_env;

    fn get_context(predecessor: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder.predecessor_account_id(predecessor);
        builder.block_timestamp(100);
        builder
    }

    #[test]
    fn test_default_index_fund() {
        let contract = IndexFund::default();
        assert!(contract.curator_address.is_none());
        assert_eq!(contract.last_rebalance, U64(0));
        assert_eq!(contract.rebalance_interval, U64(86400));
        assert_eq!(contract.get_assets().len(), 0);
    }

    #[test]
    fn test_update_weights() {
        let curator = AccountId::from_str("curator.near").unwrap();
        let asset1 = AccountId::from_str("asset1.near").unwrap();
        let asset2 = AccountId::from_str("asset2.near").unwrap();

        let context = get_context(curator.clone());
        testing_env!(context.build());

        let mut contract = IndexFund::default();
        contract.curator_address = Some(curator);

        let updates = vec![
            AssetWeight {
                weight: U64(6000),
                asset_address: asset1.clone(),
            },
            AssetWeight {
                weight: U64(4000),
                asset_address: asset2.clone(),
            },
        ];

        contract.update_weights(updates);

        let weights = contract.get_weights();
        assert_eq!(weights.len(), 2);

        let asset1_weight = weights
            .iter()
            .find(|w| w.asset_address == asset1)
            .expect("Asset1 not found");
        assert_eq!(asset1_weight.weight, U64(6000));

        let asset2_weight = weights
            .iter()
            .find(|w| w.asset_address == asset2)
            .expect("Asset2 not found");
        assert_eq!(asset2_weight.weight, U64(4000));
    }

    #[test]
    #[should_panic(expected = "curator not registered")]
    fn test_update_weights_without_curator() {
        let mut contract = IndexFund::default();
        let asset = AccountId::from_str("asset.near").unwrap();

        contract.update_weights(vec![AssetWeight {
            weight: U64(10000),
            asset_address: asset,
        }]);
    }

    #[test]
    #[should_panic(expected = "Unauthorized")]
    fn test_update_weights_unauthorized() {
        let curator = AccountId::from_str("curator.near").unwrap();
        let unauthorized = AccountId::from_str("unauthorized.near").unwrap();
        let asset = AccountId::from_str("asset.near").unwrap();

        let context = get_context(unauthorized);
        testing_env!(context.build());

        let mut contract = IndexFund::default();
        contract.curator_address = Some(curator);

        contract.update_weights(vec![AssetWeight {
            weight: U64(10000),
            asset_address: asset,
        }]);
    }

    #[test]
    #[should_panic(expected = "Final weights must sum to 100%")]
    fn test_update_weights_invalid_sum() {
        let curator = AccountId::from_str("curator.near").unwrap();
        let asset = AccountId::from_str("asset.near").unwrap();

        let context = get_context(curator.clone());
        testing_env!(context.build());

        let mut contract = IndexFund::default();
        contract.curator_address = Some(curator);

        contract.update_weights(vec![AssetWeight {
            weight: U64(5000), // Only 50% instead of required 100%
            asset_address: asset,
        }]);
    }
}