contract;

use std::{
    bytes::Bytes,
    context::msg_amount,
    contract_id::ContractId,
    revert::revert,
    storage::storage_vec::*,
};

use sway_libs::ownership::*;
use standards::src5::State;
use std_hook_metadata::*;
use interfaces::{hooks::{aggregation_hook::*, post_dispatch_hook::*,}, ownable::Ownable,};

storage {
    /// The list of hooks to aggregate
    hooks: StorageVec<ContractId> = StorageVec {},
}

impl AggregationHook for Contract {
    /// Initializes the AggregationHook contract.
    ///
    /// ### Arguments
    ///
    /// * `owner`: [b256] - The owner of the contract.
    /// * `hooks`: [Vec<ContractId>] - The hooks to initialize with.
    ///
    /// ### Reverts
    ///
    /// * If the contract is already initialized.
    #[storage(write)]
    fn initialize(owner: b256, hooks: Vec<ContractId>) {
        initialize_ownership(Identity::Address(Address::from(owner)));
        let mut i = 0;
        while i < hooks.len() {
            storage.hooks.push(hooks.get(i).unwrap());
            i += 1;
        }
    }

    /// Returns the hooks.
    ///
    /// ### Returns
    ///
    /// * `hooks`: [Vec<ContractId>] - The hooks.
    #[storage(read)]
    fn get_hooks() -> Vec<ContractId> {
        storage.hooks.load_vec()
    }
}

impl PostDispatchHook for Contract {
    /// Returns an enum that represents the type of hook
    ///
    /// ### Returns
    ///
    /// * [PostDispatchHookType] - The type of the hook.
    fn hook_type() -> PostDispatchHookType {
        PostDispatchHookType::AGGREGATION
    }

    /// Returns whether the hook supports metadata
    ///
    /// ### Arguments
    ///
    /// * `metadata`: [Bytes] - The metadata to be checked.
    ///
    /// ### Returns
    ///
    /// * [bool] - Whether the hook supports the metadata.
    #[storage(read)]
    fn supports_metadata(metadata: Bytes) -> bool {
        StandardHookMetadata::is_valid(metadata)
    }

    /// Compute the payment required by the postDispatch call
    ///
    /// ### Arguments
    ///
    /// * `metadata`: [Bytes] - The metadata required for the hook.
    /// * `message`: [Bytes] - The message being dispatched.
    ///
    /// ### Returns
    ///
    /// * [u64] - The payment required for the postDispatch call.
    ///
    /// ### Reverts
    ///
    /// * If the contract is not initialized.
    #[storage(read)]
    fn quote_dispatch(metadata: Bytes, message: Bytes) -> u64 {
        let hooks = storage.hooks.load_vec();

        let mut total = 0;
        let mut i = 0;
        while i < hooks.len() {
            let hook = hooks.get(i).unwrap();
            total += _hook_qoute_dispatch(hook, metadata, message);
            i += 1;
        }
        total
    }

    /// Executes the postDispatch call on all hooks
    ///
    /// ### Arguments
    ///
    /// * `metadata`: [Bytes] - The metadata required for the hook.
    /// * `message`: [Bytes] - The message being dispatched.
    ///
    /// ### Reverts
    ///
    /// * If the contract is not initialized.
    #[payable]
    #[storage(read, write)]
    fn post_dispatch(metadata: Bytes, message: Bytes) {
        let hooks = storage.hooks.load_vec();

        let mut i = 0;
        while i < hooks.len() {
            let hook = hooks.get(i).unwrap();
            let quote = _hook_qoute_dispatch(hook, metadata, message);

            let hook_contract = abi(PostDispatchHook, hook.bits());
            hook_contract
                .post_dispatch {
                    asset_id: b256::from(AssetId::base()),
                    coins: quote,
                }(metadata.clone(), message.clone());

            i += 1;
        }
    }
}

// ------------------------------------------------------------
// ------------------ Internal Functions ----------------------
// ------------------------------------------------------------



fn _hook_qoute_dispatch(hook: ContractId, metadata: Bytes, message: Bytes) -> u64 {
    let hook_contract = abi(PostDispatchHook, hook.bits());
    hook_contract.quote_dispatch(metadata.clone(), message.clone())
}
