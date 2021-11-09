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
use near_sdk::json_types::{U64, U128, ValidAccountId};
use near_contract_standards::fungible_token::metadata::{FungibleTokenMetadata};
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use utils::get_claim_amount;

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
    tokens: UnorderedMap<AccountId, LockupInfo>,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct LockupInfo {
    tasks: Vector<Task>,
    balance: u128,
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
    fn on_claim(&mut self, token_id: AccountId, task_index: u32, claimer_id: AccountId, amount: U128);

    fn on_add_token(&mut self, token_id: AccountId);
}

#[near_bindgen]
impl Lockup {
    #[init]
    pub fn new(owner_id: AccountId) -> Self {
        Self {
            owner_id,
            tokens: UnorderedMap::new(b't')
        }
    }

    #[payable]
    pub fn add_token(&mut self, token_id: AccountId) {
        let sender = env::predecessor_account_id();
        assert!(sender == self.owner_id, "contract owner only");
        self.internal_add_token(token_id.clone());
    }

    pub fn add_task(&mut self, token_id: AccountId, account_list: Vec<AccountId>, start_time: U64, end_time: U64, vesting_period: U64, amount: U128) {
        assert!(self.owner_id == env::predecessor_account_id(), "contract owner only");
        let mut token = self.tokens.get(&token_id).unwrap();
        let start_time: u64 = start_time.into();
        let end_time: u64 = end_time.into();
        let amount: u128 = amount.into();
        let vesting_period: u64 = vesting_period.into();
        assert!(token.balance >= amount, "not enough balance");
        assert!(start_time < end_time, "start time should larger than end time");
        assert!(end_time - start_time >= vesting_period, "total duration must be larger than a single vesting period");
        assert!(account_list.len() > 0, "list length should greater than 0");
        let single_account_amount = amount / account_list.len() as u128;
        let claim_count = ((end_time - start_time) / vesting_period) as u128;
        let single_claim_amount = single_account_amount / claim_count;
        
        let key_prefix = token_id.clone() + &token.tasks.len().to_string();
        let mut accounts = LookupMap::new(key_prefix.into_bytes());
        for account in account_list {
            accounts.insert(&account, &ClaimInfo {
                amount_left: single_account_amount,
                claim_time: start_time
            });
        }
        token.balance -= amount;
        token.tasks.push(&Task { 
            token_id: token_id.clone(), 
            accounts: accounts, 
            start_time: start_time.into(), 
            end_time: end_time.into(), 
            vesting_period: vesting_period.into(), 
            amount: amount.into(),
            single_claim_amount: single_claim_amount,
        });
        self.tokens.insert(&token_id, &token);
    }

    pub fn claim(&mut self, token_id: AccountId, task_index: u32) {
        let sender = env::predecessor_account_id();
        let task = self.tokens.get(&token_id).unwrap().tasks.get(task_index as u64).unwrap();
        assert!(task.accounts.get(&sender).is_some(), "not allowed to claim");
        let claim_info = task.accounts.get(&sender).unwrap();
        let amount = get_claim_amount(&task, &claim_info);
        if u128::from(amount) > 0  {
            ext_fungible_token::ft_transfer(sender.clone(), amount.into(), None, &token_id, 1, env::prepaid_gas() / 3).then(
                ext_self::on_claim(token_id, task_index, sender, amount, &env::current_account_id(), 0, env::prepaid_gas() / 3)
            );
        }
        
    }
}

/*
 * The rest of this file holds the inline tests for the code above
 * Learn more about Rust tests: https://doc.rust-lang.org/book/ch11-01-writing-tests.html
 *
 * To run from contract directory:
 * cargo test -- --nocapture
 *
 * From project root, to run in combination with frontend tests:
 * yarn test
 *
 */
#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, VMContext};

    // mock the context for testing, notice "signer_account_id" that was accessed above from env::
    fn get_context(input: Vec<u8>, is_view: bool) -> VMContext {
        VMContext {
            current_account_id: "alice_near".to_string(),
            signer_account_id: "bob_near".to_string(),
            signer_account_pk: vec![0, 1, 2],
            predecessor_account_id: "carol_near".to_string(),
            input,
            block_index: 0,
            block_timestamp: 0,
            account_balance: 0,
            account_locked_balance: 0,
            storage_usage: 0,
            attached_deposit: 0,
            prepaid_gas: 10u64.pow(18),
            random_seed: vec![0, 1, 2],
            is_view,
            output_data_receivers: vec![],
            epoch_height: 19,
        }
    }

    #[test]
    fn set_then_get_greeting() {
        let context = get_context(vec![], false);
        testing_env!(context);
        let mut contract = Lockup::default();
    }
}
