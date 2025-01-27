contract;

use std::{bytes::Bytes, hash::*, storage::storage_vec::*};
use standards::src5::State;
use sway_libs::{ownership::*};
use interfaces::{isms::{ism::*, routing::{domain_routing_ism::*, routing_ism::*}}, ownable::*};
use message::{EncodedMessage, Message};

/// Errors that can occur in the DomainRoutingIsm.
enum DomainRoutingIsmError {
    AlreadyInitialized:(),
    NotInitialized:(),
    DomainModuleLengthMismatch:(u64, u64),
    DomainNotSet:(u32),
}

storage {
    /// Mapping of modules which are used for specific domains.
    domain_modules: StorageMap<u32, b256> = StorageMap {},
    /// List of domains that have been set.
    domains: StorageVec<u32> = StorageVec {},
}

impl InterchainSecurityModule for Contract {
    /// Returns an enum that represents the type of security model
    /// encoded by this ISM. Relayers infer how to fetch and format metadata.
    ///
    /// ### Returns
    ///
    /// * [ModuleType] - The type of security model.
    fn module_type() -> ModuleType {
        ModuleType::ROUTING
    }

    /// Verifies the message using the metadata.
    ///
    /// ### Arguments
    ///
    /// * `metadata`: [Bytes] - The metadata to be used for verification.
    /// * `message`: [Bytes] - The message to be verified.
    ///
    /// ### Returns
    ///
    /// * [bool] - True if the message is verified successfully.
    ///
    /// ### Reverts
    ///
    /// * If the ISM is not initialized.
    /// * If the ISM call fails.
    #[storage(read)]
    fn verify(metadata: Bytes, message: Bytes) -> bool {
        only_initialized();

        let ism_id = _route(message);
        let ism = abi(InterchainSecurityModule, ism_id);
        ism.verify(metadata, message)
    }
}

impl RoutingIsm for Contract {
    /// Returns the ISM responsible for verifying the message.
    ///
    /// ### Arguments
    ///
    /// * `message`: [Bytes] - Formatted Hyperlane message
    ///
    /// ### Returns
    ///
    /// * [b256] - The ISM to use to verify the message
    ///
    /// ### Reverts
    ///
    /// * If the ISM is not initialized.
    /// * If the domain is not set.
    #[storage(read)]
    fn route(message: Bytes) -> b256 {
        only_initialized();
        _route(message)
    }
}

impl DomainRoutingIsm for Contract {
    /// Sets the owner of the ISM.
    ///
    /// ### Arguments
    ///
    /// * `owner`: [b256] - The address of the owner.
    ///
    /// ### Reverts
    ///
    /// * If the ISM is already initialized.
    #[storage(write, read)]
    fn initialize(owner: b256){
        initialize_ownership(Identity::Address(Address::from(owner)));
    }

    /// Sets the ISMs to be used for the specified origin domains
    ///
    /// ### Arguments
    ///
    /// * `owner`: [b256] - The address of the owner.
    /// * `domains`: [Vec<u32>] - The list of origin domains.
    /// * `modules`: [Vec<b256>] - The list of ISMs to be used for the specified domains.
    ///
    /// ### Reverts
    ///
    /// * If the ISM is already initialized.
    /// * If the length of the domains and modules do not match.
    #[storage(write, read)]
    fn initialize_with_domains(owner: b256, domains: Vec<u32>, modules: Vec<b256>) {

        initialize_ownership(Identity::Address(Address::from(owner)));
        let domain_count = domains.len();
        let module_count = modules.len();
        require(
            domain_count == module_count,
            DomainRoutingIsmError::DomainModuleLengthMismatch((domain_count, module_count)),
        );

        let mut domains = domains;
        let mut modules = modules;

        while true {
            let domain = domains.pop();
            let module = modules.pop();
            if domain.is_some() && module.is_some() {
                _set(domain.unwrap(), module.unwrap());
            } else {
                break;
            }
        }
    }

    /// Sets the ISM to be used for the specified origin domain
    ///
    /// ### Arguments
    ///
    /// * `domain`: [u32] - The origin domain.
    /// * `module`: [b256] - The ISM to be used for the specified domain.
    ///
    /// ### Reverts
    ///
    /// * If the ISM is not initialized.
    /// * If the caller is not the owner.
    #[storage(write, read)]
    fn set(domain: u32, module: b256) {
        only_initialized();
        only_owner();

        _set(domain, module);
    }   

    /// Removes the specified origin domain
    ///
    /// ### Arguments
    ///
    /// * `domain`: [u32] - The origin domain.
    ///
    /// ### Reverts
    ///
    /// * If the ISM is not initialized.
    /// * If the caller is not the owner.
    #[storage(write, read)]
    fn remove(domain: u32) {
        only_initialized();
        only_owner();

        let success = storage.domain_modules.remove(domain);
        if success {
            _remove_domain(domain);
        }
    }

    /// Returns the domains that have been set
    ///
    /// ### Returns
    ///
    /// * [Vec<u32>] - The list of origin domains.
    #[storage(read)]
    fn domains() -> Vec<u32> {
        storage.domains.load_vec()
    }

    /// Returns the ISM to be used for the specified origin domain
    ///
    /// ### Arguments
    ///
    /// * `domain`: [u32] - The origin domain.
    ///
    /// ### Returns
    ///
    /// * [b256] - The ISM to be used for the specified domain.
    #[storage(read)]
    fn module(domain: u32) -> b256 {
        storage.domain_modules.get(domain).try_read().unwrap_or(b256::zero())
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
        initialize_ownership(new_owner);
    }

    #[storage(read, write)]
    fn renounce_ownership() {
        renounce_ownership();
    }
}


// --- Internal functions ---

#[storage(read)]
fn _route(message: Bytes) -> b256 {
    let domain = EncodedMessage::from_bytes(message).origin();
    let module = storage.domain_modules.get(domain).try_read().unwrap_or(b256::zero());
    require(
        module != b256::zero(),
        DomainRoutingIsmError::DomainNotSet(domain),
    );
    module
        
}

#[storage(read)]
fn _domain_exists(domain: u32) -> bool {
    let domains = storage.domains.load_vec();

    for d in domains.iter() {
        if d == domain {
            return true;
        }
    }
    return false;
}

#[storage(read, write)]
fn _remove_domain(domain: u32) {
    let domains = storage.domains.load_vec();
    let mut index = 0;
    for d in domains.iter() {
        if d == domain {
            let _ = storage.domains.remove(index);
            return;
        }
        index += 1;
    }
}

#[storage(write, read)]
fn _set(domain: u32, module: b256) {
    if !_domain_exists(domain) {
        storage.domains.push(domain);
    }
    storage.domain_modules.insert(domain, module);
}

// --- Guards ---

#[storage(read)]
fn only_initialized() {
    require(
        _owner() != State::Uninitialized,
        DomainRoutingIsmError::NotInitialized,
    );
}