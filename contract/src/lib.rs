/*
 * This is an example of a Rust smart contract with two simple, symmetric functions:
 *
 * 1. set_greeting: accepts a greeting, such as "howdy", and records it for the user (account_id)
 *    who sent the request
 * 2. get_greeting: accepts an account_id and returns the greeting saved for it, defaulting to
 *    "Hello"
 *
 * Learn more about writing NEAR smart contracts with Rust:
 * https://github.com/near/near-sdk-rs
 *
 */

// To conserve gas, efficient serialization is achieved through Borsh (http://borsh.io/)
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{AccountId, PanicOnDefault, env, ext_contract, log, near_bindgen, setup_alloc};
use near_sdk::collections::{LookupMap, UnorderedMap, Vector};
use near_sdk::serde::{Serialize, Deserialize};
use near_sdk::json_types::{Base58CryptoHash, U128, U64, ValidAccountId};
use near_contract_standards::fungible_token::metadata::{FungibleTokenMetadata};
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use utils::get_claim_amount;
use std::convert::TryInto;

setup_alloc!();

pub mod internal;
pub mod utils;
pub mod view;

// Structs in Rust are similar to other languages, and may include impl keyword as shown below
// Note: the names of the structs are not important when calling the smart contract, but the function names are
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Lockup {
    owner_id: AccountId,
    tokens: Vector<AccountId>,
    tasks: UnorderedMap<Base58CryptoHash, Task>,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Task {
    token_id: AccountId,
    accounts: LookupMap<AccountId, ClaimInfo>,
    start_time: u64,
    end_time: u64,
    vesting_period: u64,
    amount: u128,
    single_claim_amount: u128,
}

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
#[derive(Debug, Clone)]
pub struct ClaimInfo {
    amount_left: u128,
    claim_time: u64,
}

#[ext_contract(ext_fungible_token)]
pub trait FungibleTokenContract {
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>);

    fn ft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<U128>;

    fn ft_metadata() -> FungibleTokenMetadata;

    fn storage_deposit(
        &mut self,
        account_id: Option<ValidAccountId>,
        registration_only: Option<bool>,
    ) -> StorageBalance;
}

#[ext_contract(ext_self)]
pub trait ExtSelf {
    fn on_claim(&mut self, hash: Base58CryptoHash, claimer_id: AccountId, amount: U128);

    fn on_add_token(&mut self, token_id: AccountId);
}

#[near_bindgen]
impl Lockup {
    #[init]
    pub fn new(owner_id: AccountId) -> Self {
        Self {
            owner_id,
            tokens: Vector::new(b't'),
            tasks: UnorderedMap::new(b'a')
        }
    }

    #[payable]
    pub fn add_token(&mut self, token_id: AccountId) {
        let sender = env::predecessor_account_id();
        assert!(sender == self.owner_id, "contract owner only");
        self.internal_add_token(token_id);
    }

    pub fn claim(&mut self, token_id: AccountId, hash: Base58CryptoHash) {
        let sender = env::predecessor_account_id();
        let task = self.tasks.get(&hash).unwrap();
        assert!(task.accounts.get(&sender).is_some(), "not allowed to claim");
        let claim_info = task.accounts.get(&sender).unwrap();
        let amount = get_claim_amount(&task, &claim_info);
        if u128::from(amount) > 0  {
            ext_fungible_token::ft_transfer(sender.clone(), amount.into(), None, &token_id, 1, env::prepaid_gas() / 3).then(
                ext_self::on_claim(hash, sender, amount, &env::current_account_id(), 0, env::prepaid_gas() / 3)
            );
        }
        
    }
}