
use near_sdk::{log};

use crate::*;

pub(crate) fn get_claim_amount(task: &Task, claim_info: &ClaimInfo) -> U128 {
    let claim_duration = claim_info.claim_time - task.start_time;
    let claim_count = claim_duration / task.vesting_period;

    let present_duration = env::block_timestamp() - task.start_time;
    let present_claim_count = present_duration / task.vesting_period;

    let count = present_claim_count - claim_count;
    let mut claim_amount = count  as u128 * task.single_claim_amount;

    if env::block_timestamp() >= task.end_time{
        claim_amount = claim_info.amount_left;
    }
    claim_amount.into()
}