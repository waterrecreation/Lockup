


use crate::*;
use near_sdk::{PromiseOrValue, PromiseResult, json_types::ValidAccountId, serde_json};

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
#[derive(Debug, Clone)]
pub struct TaskArgs {
    token_id: AccountId, 
    account_list: Vec<AccountId>, 
    start_time: U64, 
    end_time: U64, 
    vesting_period: U64, 
    amount: U128
}

impl Lockup {

    pub(crate) fn internal_add_task(&mut self, task_args: TaskArgs, token_amount: u128) -> U128{
        let mut token_amount = token_amount;
        let start_time: u64 = task_args.start_time.into();
        let end_time: u64 = task_args.end_time.into();
        let amount: u128 = task_args.amount.into();
        let vesting_period: u64 = task_args.vesting_period.into();
        assert!(token_amount >= amount, "not enough balance");
        assert!(start_time < end_time, "start time should larger than end time");
        assert!(end_time - start_time >= vesting_period, "total duration must be larger than a single vesting period");
        assert!(task_args.account_list.len() > 0, "list length should greater than 0");
        let single_account_amount = amount / task_args.account_list.len() as u128;
        let claim_count = ((end_time - start_time) / vesting_period) as u128;
        let single_claim_amount = single_account_amount / claim_count;
        
        let key_prefix = task_args.token_id.clone() + &self.tasks.len().to_string();
        let mut accounts = LookupMap::new(key_prefix.into_bytes());
        for account in task_args.account_list {
            accounts.insert(&account, &ClaimInfo {
                amount_left: single_account_amount,
                claim_time: start_time
            });
        }
        token_amount -= amount;
        self.tasks.push(&Task { 
            token_id: task_args.token_id.clone(), 
            accounts: accounts, 
            start_time: start_time.into(), 
            end_time: end_time.into(), 
            vesting_period: vesting_period.into(), 
            amount: amount.into(),
            single_claim_amount: single_claim_amount,
        });
        token_amount.into()
    }

    pub(crate) fn internal_add_token(&mut self, token_id: AccountId) {
        assert!(self.tokens.iter().find(|token| *token == token_id).is_none(), "token already exist");
        ext_fungible_token::storage_deposit(Some(env::current_account_id().try_into().unwrap()), None, &token_id, env::attached_deposit(), env::prepaid_gas() / 3).then(
            ext_self::on_add_token(token_id.clone(), &env::current_account_id(), 0, env::prepaid_gas() / 3)
        );
    }

}

#[near_bindgen]
impl Lockup {
    #[private]
    pub fn on_claim(&mut self, index: u32, claimer_id: AccountId, amount: U128) {
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {
                let mut task = self.tasks.get(index as u64).unwrap();
                let mut claim_info = task.accounts.get(&claimer_id).unwrap();
                claim_info.amount_left -= u128::from(amount);
                claim_info.claim_time = env::block_timestamp();
                task.accounts.insert(&claimer_id, &claim_info);
            },
            PromiseResult::Failed => {
                log!("failed to claim");
            }
        }
    }

    #[private]
    pub fn on_add_token(&mut self, token_id: AccountId) {
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {
                self.tokens.push(&token_id);
            },
            PromiseResult::Failed => {
                log!("failed to claim");
            }
        }
    }
}

#[near_bindgen]
#[allow(unreachable_code)]
impl FungibleTokenReceiver for Lockup {
    /// Callback on receiving tokens by this contract.
    /// `msg` format is either "" for deposit or `TokenReceiverMessage`.
    fn ft_on_transfer(
        &mut self,
        sender_id: ValidAccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        assert!(!msg.is_empty(), "msg should not be empty");
        let token_in = env::predecessor_account_id();
        let task_args: TaskArgs = serde_json::from_str(&msg).unwrap();
        assert!(task_args.token_id == token_in, "token not match");
        assert!(self.owner_id == sender_id.to_string(), "contract owner only");
        let amount_left = self.internal_add_task(task_args, amount.into());
        PromiseOrValue::Value(amount_left)
    }
}