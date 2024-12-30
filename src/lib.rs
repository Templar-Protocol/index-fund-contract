// Find all our documentation at https://docs.near.org
use near_sdk::{log, near};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap};
use near_sdk::json_types::U128;
use near_sdk::{env, near_bindgen, AccountId, Balance, Promise, require};

pub type AssetId = AccountId;
pub type Price = U128;

#[near(serializers = [json, borsh])]
pub struct AssetWeight {
    pub weight: u32,  // Basis points (e.g., 5000 = 50%)
    pub asset_address: AssetId,
}

#[near(serializers = [json, borsh])]
pub struct AssetHolding {
    pub balance: Balance,
    pub weight: u32,
    pub last_price: Price,
    pub last_updated: u64,
}

#[near_bindgen]
#[near(serializers = [json, borsh])]
pub struct IndexFund {
    pub dao_address: Option<AccountId>,
    pub assets: UnorderedMap<AssetId, AssetHolding>,
    pub last_rebalance: u64,
    pub rebalance_interval: u64,
    pub lp_token: AccountId,
    pub oracle: AccountId,
}

// Define the contract structure
#[near(contract_state)]
pub struct Contract {
    greeting: String,
}

// Define the default, which automatically initializes the contract
impl Default for Contract {
    fn default() -> Self {
        Self {
            greeting: "Hello".to_string(),
        }
    }
}

// Implement the contract structure
#[near]
impl Contract {
    // Public method - returns the greeting saved, defaulting to DEFAULT_GREETING
    pub fn get_greeting(&self) -> String {
        self.greeting.clone()
    }

    // Public method - accepts a greeting, such as "howdy", and records it
    pub fn set_greeting(&mut self, greeting: String) {
        log!("Saving greeting: {greeting}");
        self.greeting = greeting;
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
