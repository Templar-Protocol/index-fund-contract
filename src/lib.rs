// Find all our documentation at https://docs.near.org
use near_sdk::{near, NearToken};
use near_sdk::collections::{UnorderedMap};
use near_sdk::{env, AccountId, require};
use near_sdk::json_types::{U64, U128};

use std::str::FromStr;

pub type AssetId = AccountId;
pub type Price = U128;

#[derive(Debug)]
#[near(serializers = [json, borsh])]
pub struct AssetWeight {
    pub weight: u32,  // Basis points (e.g., 5000 = 50%)
    pub asset_address: AssetId,
}

#[near(serializers = [json, borsh])]
pub struct AssetHolding {
    pub balance: U128,
    pub weight: u32,
    pub last_price: U128,
    pub last_updated: U64,
}

// Define the contract structure
#[near(contract_state)]
pub struct IndexFund {
    pub dao_address: Option<AccountId>,
    pub assets: UnorderedMap<AssetId, AssetHolding>,
    pub last_rebalance: U64,
    pub rebalance_interval: U64, // blocks
}

impl Default for IndexFund {
    fn default() -> Self {
        Self {
            dao_address: None,
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
    pub fn new(rebalance_interval: u64) -> Self {
        require!(rebalance_interval > 0, "Invalid rebalance interval");
        Self {
            dao_address: None,
            assets: UnorderedMap::new(b"a"),
            last_rebalance: U64(0),
            rebalance_interval: U64(rebalance_interval),
        }
    }

    #[payable]
    pub fn register_dao(&mut self, dao_address: AccountId) {
        require!(self.dao_address.is_none(), "DAO already registered");
        require!(
            env::attached_deposit() >= NearToken::from_yoctonear(env::storage_byte_cost().as_yoctonear() * 100),
            "Insufficient storage deposit"
        );
        self.dao_address = Some(dao_address);
    }

    pub fn update_weights(&mut self, updates: Vec<AssetWeight>) {
        let dao = self.dao_address.as_ref().expect("DAO not registered");
        require!(env::predecessor_account_id() == *dao, "Unauthorized");
        
        let total_weight: u32 = updates.iter().map(|u| u.weight).sum();
        require!(total_weight == 10000, "Weights must sum to 100%");

        for update in updates.iter() {
            if let Some(mut holding) = self.assets.get(&update.asset_address) {
                holding.weight = update.weight;
                self.assets.insert(&update.asset_address, &holding);
            } else {
                self.assets.insert(&update.asset_address, &AssetHolding {
                    balance: U128(0),
                    weight: update.weight,
                    last_price: U128(0),
                    last_updated: U64(env::block_timestamp()),
                });
            }
        }

        env::log_str(&format!("Updated weights: {:?}", updates));
    }

    pub fn get_weights(&self) -> Vec<AssetWeight> {
        self.assets.iter().map(|(asset_address, h)| AssetWeight {
            weight: h.weight,
            asset_address,
        }).collect()
    }

    pub fn get_assets(&self) -> Vec<AssetId> {
        self.assets.keys().collect()
    }

}

/*
 * The rest of this file holds the inline tests for the code above
 * Learn more about Rust tests: https://doc.rust-lang.org/book/ch11-01-writing-tests.html
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
        assert!(contract.dao_address.is_none());
        assert_eq!(contract.last_rebalance, U64(0));
        assert_eq!(contract.rebalance_interval, U64(86400));
        assert_eq!(contract.get_assets().len(), 0);
    }

    #[test]
    fn test_update_weights() {
        let dao = AccountId::from_str("dao.near").unwrap();
        let asset1 = AccountId::from_str("asset1.near").unwrap();
        let asset2 = AccountId::from_str("asset2.near").unwrap();
        
        let context = get_context(dao.clone());
        testing_env!(context.build());

        let mut contract = IndexFund::default();
        contract.dao_address = Some(dao);

        let updates = vec![
            AssetWeight {
                weight: 6000,
                asset_address: asset1.clone(),
            },
            AssetWeight {
                weight: 4000,
                asset_address: asset2.clone(),
            },
        ];

        contract.update_weights(updates);

        let weights = contract.get_weights();
        assert_eq!(weights.len(), 2);
        
        let asset1_weight = weights.iter()
            .find(|w| w.asset_address == asset1)
            .expect("Asset1 not found");
        assert_eq!(asset1_weight.weight, 6000);

        let asset2_weight = weights.iter()
            .find(|w| w.asset_address == asset2)
            .expect("Asset2 not found");
        assert_eq!(asset2_weight.weight, 4000);
    }

    #[test]
    #[should_panic(expected = "DAO not registered")]
    fn test_update_weights_without_dao() {
        let mut contract = IndexFund::default();
        let asset = AccountId::from_str("asset.near").unwrap();
        
        contract.update_weights(vec![AssetWeight {
            weight: 10000,
            asset_address: asset,
        }]);
    }

    #[test]
    #[should_panic(expected = "Unauthorized")]
    fn test_update_weights_unauthorized() {
        let dao = AccountId::from_str("dao.near").unwrap();
        let unauthorized = AccountId::from_str("unauthorized.near").unwrap();
        let asset = AccountId::from_str("asset.near").unwrap();
        
        let context = get_context(unauthorized);
        testing_env!(context.build());

        let mut contract = IndexFund::default();
        contract.dao_address = Some(dao);

        contract.update_weights(vec![AssetWeight {
            weight: 10000,
            asset_address: asset,
        }]);
    }

    #[test]
    #[should_panic(expected = "Weights must sum to 100%")]
    fn test_update_weights_invalid_sum() {
        let dao = AccountId::from_str("dao.near").unwrap();
        let asset = AccountId::from_str("asset.near").unwrap();
        
        let context = get_context(dao.clone());
        testing_env!(context.build());

        let mut contract = IndexFund::default();
        contract.dao_address = Some(dao);

        contract.update_weights(vec![AssetWeight {
            weight: 5000,  // Only 50% instead of required 100%
            asset_address: asset,
        }]);
    }
}
