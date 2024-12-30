// Find all our documentation at https://docs.near.org
use near_sdk::{log, near, NearToken};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{UnorderedMap};
use near_sdk::{env, AccountId, Promise, require};
use near_sdk::json_types::{U64, U128};

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

    #[test]
    fn get_default_greeting() {
        let contract = Contract::default();
        // this test did not call set_greeting so should return the default "Hello" greeting
        assert_eq!(contract.get_greeting(), "Hello");
    }

    #[test]
    fn set_then_get_greeting() {
        let mut contract = Contract::default();
        contract.set_greeting("howdy".to_string());
        assert_eq!(contract.get_greeting(), "howdy");
    }
}
