use crate::*;

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
#[derive(Debug, Clone)]
pub struct TaskInfo {
    token_id: AccountId,
    index: u32,
    start_time: U64,
    end_time: U64,
    vesting_period: U64,
    amount_left: U128,
    claim_time: U64,
    should_claim: U128 
}

impl Lockup {
  pub(crate) fn internal_get_task_by_token_id(&self, tasks: Vec<Task>, sender: AccountId) -> Vec<TaskInfo> {
    let mut ret: Vec<TaskInfo> = Vec::new();
    for (i, task) in tasks.iter().enumerate() {
      match task.accounts.get(&sender) {
        Some(v) => {
          ret.push(TaskInfo {
            index: i as u32,
            should_claim: get_claim_amount(task, &v),
            amount_left: v.amount_left.into(),
            claim_time: v.claim_time.into(),
            token_id: task.token_id.clone(),
            start_time: task.start_time.into(),
            end_time: task.end_time.into(),
            vesting_period: task.vesting_period.into(),
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

  pub fn get_tasks(&self, sender: AccountId) -> Vec<TaskInfo> {
    let mut tasks: Vec<TaskInfo> = Vec::new();
    for (_, token) in self.tokens.iter() {
      tasks = [tasks, self.internal_get_task_by_token_id(token.tasks.to_vec(), sender.clone())].concat();
    }
    tasks
  }

  pub fn get_tasks_by_token_id(&self, token_id: AccountId, sender: AccountId) -> Vec<TaskInfo> {
    let token = self.tokens.get(&token_id).unwrap();
    self.internal_get_task_by_token_id(token.tasks.to_vec(), sender.clone())
  }
}