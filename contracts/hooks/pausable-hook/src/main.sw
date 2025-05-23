contract;

use interfaces::{
    ownable::*,
    hooks::post_dispatch_hook::*,
};
use standards::src5::State;
use sway_libs::{ownership::*, pausable::*};
use std_hook_metadata::{StandardHookMetadata};
use std::{bytes::Bytes};

configurable {
    EXPECTED_OWNER: b256 = b256::zero(),
}


impl PostDispatchHook for Contract {
    /// Returns an enum that represents the type of hook
    ///
    /// ### Returns
    ///
    /// * [PostDispatchHookType] - The type of the hook.
    fn hook_type() -> PostDispatchHookType {
        PostDispatchHookType::PAUSABLE
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
    fn supports_metadata(metadata: Bytes) -> bool {
        // We perform the same check as EVM for compatibility
        StandardHookMetadata::is_valid(metadata)
    }

    /// Post action after a message is dispatched via the Mailbox
    /// For the MerkleTreeHook, this function inserts the message ID into the MerkleTree.
    ///
    /// ### Arguments
    ///
    /// * `metadata`: [Bytes] - The metadata required for the hook.
    /// * `message`: [Bytes] - The message to be processed.
    ///
    /// ### Reverts
    ///
    /// * If the contract is paused.
    #[payable]
    #[storage(read, write)]
    fn post_dispatch(_metadata: Bytes, _message: Bytes) {
        require_not_paused();
    }

    /// Compute the payment required by the postDispatch call
    ///
    /// ### Arguments
    ///
    /// * `metadata`: [Bytes] - The metadata required for the hook.
    /// * `message`: [Bytes] - The message to be processed.
    ///
    /// ### Returns
    ///
    /// * [u64] - The payment required for the postDispatch call.
    #[storage(read)]
    fn quote_dispatch(_metadata: Bytes, _message: Bytes) -> u64 {
        0
    }
}

/// All functions required for the PausableHook Hyperlane interface are inherited through the Pausable abi.
/// Source: [sway-libs](https://github.com/FuelLabs/sway-libs/blob/master/libs/src/pausable.sw)
impl Pausable for Contract {
    #[storage(write)]
    fn pause() {
        only_owner();
        _pause();
    }

    #[storage(write)]
    fn unpause() {
        only_owner();
        _unpause();
    }

    #[storage(read)]
    fn is_paused() -> bool {
        _is_paused()
    }
}

impl Ownable for Contract {
    #[storage(read)]
    fn owner() -> State {
        _owner()
    }

    #[storage(read)]
    fn only_owner() {
        only_owner();
    }

    #[storage(write)]
    fn transfer_ownership(new_owner: Identity) {
        transfer_ownership(new_owner);
    }

    #[storage(read, write)]
    fn initialize_ownership(new_owner: Identity) {
        _is_expected_owner(new_owner);
        initialize_ownership(new_owner);
    }

    #[storage(read, write)]
    fn renounce_ownership() {
        renounce_ownership();
    }
}


// Front-run guard
fn _is_expected_owner(owner: Identity) {
    require(owner.bits() == EXPECTED_OWNER, OwnableError::UnexpectedOwner);
}
