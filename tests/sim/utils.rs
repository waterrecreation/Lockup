pub use near_sdk::json_types::{Base64VecU8, ValidAccountId, WrappedDuration, U64};
use near_sdk::serde_json::json;
use near_sdk_sim::runtime::GenesisConfig;
use near_sdk_sim::{call, view, deploy, init_simulator, ContractAccount, UserAccount};
use lockup::LockupContract;
use lockup::{*};
use near_sdk::json_types::{U128};
use near_sdk_sim::to_yocto;

pub const DEFAULT_GAS: u64 = 300_000_000_000_000;

use fungible_token::ContractContract as FtContract;

// Load in contract bytes at runtime
near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    FT_WASM_BYTES => "./out/fungible_token.wasm",
    LOCKUP_WASM_BYTES => "./out/main.wasm",
}

const FT_ID: &str = "ft";
const LOCKUP_ID: &str = "lockup";

// Register the given `user` with FT contract
pub fn register_user(user: &near_sdk_sim::UserAccount) {
    user.call(
        FT_ID.to_string(),
        "storage_deposit",
        &json!({
            "account_id": user.valid_account_id()
        })
        .to_string()
        .into_bytes(),
        near_sdk_sim::DEFAULT_GAS / 2,
        near_sdk::env::storage_byte_cost() * 125, // attached deposit
    )
    .assert_success();
}

pub fn init(
    initial_balance: u128,
) -> (UserAccount, ContractAccount<LockupContract>, ContractAccount<FtContract>, UserAccount) {
    let root = init_simulator(None);
    // uses default values for deposit and gas
    let ft = deploy!(
        // Contract Proxy
        contract: FtContract,
        // Contract account id
        contract_id: FT_ID,
        // Bytes of contract
        bytes: &FT_WASM_BYTES,
        // User deploying the contract,
        signer_account: root,
        // init method
        init_method: new_default_meta(
            root.valid_account_id(),
            initial_balance.into()
        )
    );
    let alice = root.create_user("alice".to_string(), to_yocto("10000"));
    register_user(&alice);

    let lockup = deploy!(
        contract: LockupContract,
        contract_id: LOCKUP_ID,
        bytes: &LOCKUP_WASM_BYTES,
        signer_account: root,
        init_method: new(alice.account_id())
    );

    (root, lockup, ft, alice)
}