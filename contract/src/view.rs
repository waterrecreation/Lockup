use near_sdk::json_types::Base58PublicKey;

use crate::*;

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
#[derive(Debug, Clone)]
pub struct TaskInfo {
    amount_left: U128,
    claim_time: U64,
    should_claim: U128 
}

impl Lockup {
  pub(crate) fn internal_get_tasks(&self, tasks: Vec<Task>, sender: AccountId) -> Vec<TaskInfo> {
    let mut ret: Vec<TaskInfo> = Vec::new();
    for (_, task) in tasks.iter().enumerate() {
      match task.accounts.get(&sender) {
        Some(v) => {
          ret.push(TaskInfo {
            should_claim: get_claim_amount(task, &v),
            amount_left: v.amount_left.into(),
            claim_time: v.claim_time.into(),
          })
        },
        None => continue
      }
    }
    ret
  }
}

#[near_bindgen]
impl Lockup {
  pub fn get_token_list(&self) -> Vec<AccountId> {
    self.tokens.keys().collect()
  }

  pub fn get_unclaimed_amounts(&self, sender: AccountId, public_keys: Vec<Base58CryptoHash>) -> Vec<TaskInfo> {
    let mut tasks: Vec<Task> = Vec::new();
    for key in public_keys {
      match self.tasks.get(&key) {
        Some(v) => tasks.push(v),
        None => continue
      };
    }
    self.internal_get_tasks(tasks, sender)
  }
}