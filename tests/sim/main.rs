pub mod utils;
use std::convert::TryInto;

use near_sdk::borsh::BorshSerialize;
use near_sdk::serde_json::json;
use near_sdk::{PromiseOrValue};
use near_sdk::json_types::U128;
use near_sdk_sim::account::{AccessKey, AccessKeyPermission, FunctionCallPermission};
use near_sdk_sim::near_crypto::{InMemorySigner, PublicKey};
use near_sdk_sim::to_yocto;
use near_sdk_sim::{call, view, deploy, init_simulator, ContractAccount, UserAccount, DEFAULT_GAS};

#[test]
fn simulate_add_task() {
    let (root, lockup, ft, alice) = utils::init(to_yocto("100000"));

    //let runtime = root.borrow_runtime_mut();

    let transfer_amount = to_yocto("100");
    call!(
        root,
        ft.ft_transfer(alice.valid_account_id(), transfer_amount.into(),  None),
        deposit = 1
    )
    .assert_success();

    //runtime.produce_block().unwrap();

    let result: U128 = view!(
        ft.ft_balance_of(alice.valid_account_id())
    )
    .unwrap_json();
    println!("{:?}", result);

    //runtime.produce_block().unwrap();

    call!(
        alice,
        lockup.add_token(ft.account_id()),
        near_sdk::env::storage_byte_cost() * 125,
        DEFAULT_GAS
    ).assert_success();
    
    //runtime.produce_block().unwrap();

    call!(
        alice,
        ft.ft_transfer_call(lockup.valid_account_id(), transfer_amount.into(), Option::None, "".to_string()),
        1,
        DEFAULT_GAS
    ).assert_success();

    let result: U128 = view!(
        ft.ft_balance_of(lockup.valid_account_id())
    )
    .unwrap_json();
    println!("{:?}", result);

    let bob = root.create_user("bob".to_string(), to_yocto("10000"));
    let john = root.create_user("john".to_string(), to_yocto("10000"));

    //runtime.produce_block().unwrap();

    call!(
        alice,
        lockup.add_task(ft.account_id(), vec!["bob".to_string(), "john".to_string()], 0.into(), 1050.into(), 100.into(), transfer_amount.into())
    )
    .assert_success();

    //runtime.produce_block().unwrap();
    
    // call!(
    //     bob,
    //     lockup.claim(ft.account_id(), 0)
    // )
    // .assert_success();

    // let result: U128 = view!(
    //     ft.ft_balance_of(bob.valid_account_id())
    // )
    // .unwrap_json();
    // println!("{:?}", result);
}

