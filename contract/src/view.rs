
use crate::*;

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
#[derive(Debug, Clone)]
pub struct Claim {
    amount_left: U128,
    claim_time: U64,
    should_claim: U128,
    index: u32
}

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
#[derive(Debug, Clone)]
pub struct TaskInfo {
    token_id: AccountId,
    start_time: u64,
    end_time: u64,
    vesting_period: u64,
    amount: u128,
    single_claim_amount: u128,
    index: u32
}

impl Lockup {
  pub(crate) fn internal_get_tasks(&self, sender: AccountId) -> Vec<Claim> {
    let mut ret: Vec<Claim> = Vec::new();
    for (index, task) in self.tasks.iter().enumerate() {
      match task.accounts.get(&sender) {
        Some(v) => {
          ret.push(Claim {
            should_claim: get_claim_amount(&task, &v),
            amount_left: v.amount_left.into(),
            claim_time: v.claim_time.into(),
            index: index as u32
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
  pub fn get_claim_info_by_creator(&self, account_id: AccountId) -> Vec<Claim> {
    self.internal_get_tasks(account_id)
  }

  pub fn get_task(&self, index: u32) -> TaskInfo {
    let task = self.tasks.get(index as u64).unwrap();
    return TaskInfo {
      token_id: task.token_id,
      start_time: task.start_time,
      end_time: task.end_time,
      vesting_period: task.vesting_period,
      amount: task.amount,
      single_claim_amount: task.single_claim_amount,
      index: index
    }
  }
}