
use std::convert::TryInto;

use crate::*;
use near_sdk::{Balance, PromiseOrValue, PromiseResult, json_types::ValidAccountId};

impl Lockup {

    pub(crate) fn internal_deposit(&mut self, token_id: AccountId, amount: Balance) {
        let mut token = self.tokens.get(&token_id).expect("token not exist");
        token.balance += amount;
        self.tokens.insert(&token_id, &token);
    }

    pub(crate) fn internal_add_token(&mut self, token_id: AccountId) {
        assert!(self.tokens.get(&token_id.clone()).is_none(), "token already exist.");
        ext_fungible_token::storage_deposit(Some(env::current_account_id().try_into().unwrap()), None, &token_id, env::attached_deposit(), env::prepaid_gas() / 3).then(
            ext_self::on_add_token(token_id.clone(), &env::current_account_id(), 0, env::prepaid_gas() / 3)
        );
    }

}

#[near_bindgen]
impl Lockup {
    #[private]
    pub fn on_claim(&mut self, token_id: AccountId, task_index: u32, claimer_id: AccountId, amount: U128) {
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {
                let mut token = self.tokens.get(&token_id).unwrap();
                let mut task = token.tasks.get(task_index as u64).unwrap();
                let mut claim_info = task.accounts.get(&claimer_id).unwrap();
                claim_info.amount_left -= u128::from(amount);
                claim_info.claim_time = env::block_timestamp();
                task.accounts.insert(&claimer_id, &claim_info);
                self.tokens.insert(&token_id, &token);
            },
            PromiseResult::Failed => {
                log!("failed to claim");
            }
        }
    }

    #[private]
    pub fn on_add_token(
        &mut self,
        token_id: AccountId,
    ) {
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {
                self.tokens.insert(&token_id, &LockupInfo {
                    tasks: Vector::new(("t".to_string() + &token_id).into_bytes()),
                    balance: 0,
                });
            },
            PromiseResult::Failed => {
                log!("failed to add token");
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
        let token_in = env::predecessor_account_id();
        if msg.is_empty() {
            self.internal_deposit(token_in, amount.into());
            PromiseOrValue::Value(U128(0))
        } else {
            PromiseOrValue::Value(U128(0))
        }
    }
}