contract;

use sway_libs::{
    asset::{
        base::{
            _decimals,
            _name,
            _set_decimals,
            _set_name,
            _set_symbol,
            _symbol,
            _total_assets,
            _total_supply,
        },
        supply::{
            _burn,
            _mint,
        },
    },
    ownership::*,
    pausable::*,
    reentrancy::reentrancy_guard,
};

use std::{
    asset::transfer,
    bytes::Bytes,
    call_frames::msg_asset_id,
    constants::ZERO_B256,
    context::{
        msg_amount,
        this_balance,
    },
    contract_id::ContractId,
    convert::Into,
    hash::*,
    revert::revert,
    storage::storage_map::*,
    storage::storage_string::*,
    string::String,
    u128::U128,
};

use interfaces::{mailbox::mailbox::*, ownable::Ownable, warp_route::*};
use standards::src5::State;
use message::{EncodedMessage, Message};

storage {
    /// The mode of the WarpRoute contract
    token_mode: WarpRouteTokenMode = WarpRouteTokenMode::BRIDGED, // Default mode is Bridged
    /// The address of the mailbox contract to use for message dispatch
    mailbox: ContractId = ContractId::zero(),
    /// The address of the default hook contract to use for message dispatch
    default_hook: ContractId = ContractId::zero(),
    /// Mapping of message IDs to whether they have been delivered
    delivered_messages: StorageMap<b256, bool> = StorageMap {},
    // TOKEN
    /// The asset ID of the token managed by the WarpRoute contract
    asset_id: AssetId = AssetId::zero(),
    /// The sub ID of the token managed by the WarpRoute contract
    sub_id: SubId = SubId::zero(),
    /// The total number of unique assets minted by this contract.
    total_assets: u64 = 0, 
    /// The current total number of coins minted for a particular asset.
    total_supply: StorageMap<AssetId, u64> = StorageMap {}, 
    /// The mapping of asset ID to the name of the token
    name: StorageMap<AssetId, StorageString> = StorageMap {},
    /// The mapping of asset ID to the symbol of the token
    symbol: StorageMap<AssetId, StorageString> = StorageMap {},
    /// The mapping of asset ID to the number of decimals of the token
    decimals: StorageMap<AssetId, u8> = StorageMap {},
    /// The total number of coins ever minted for an asset.
    cumulative_supply: StorageMap<AssetId, u64> = StorageMap {}, 
}

configurable {
    /// The maximum supply allowed for any single asset.
    MAX_SUPPLY: u64 = 100_000_000_000_000,
}

impl WarpRoute for Contract {
     /// Initializes the WarpRoute contract
    ///
    /// ### Arguments
    ///
    /// * `owner`: [b256] - The address of the owner of the contract
    /// * `mailbox_address`: [b256] - The address of the mailbox contract to use
    /// * `mode`: [WarpRouteTokenMode] - The mode of the WarpRoute contract
    /// * `hook`: [b256] - The address of the post dispatch hook contract to use
    /// * `token_name`: [string] - The name of the token
    /// * `token_symbol`: [string] - The symbol of the token
    /// * `decimals`: [u8] - The number of decimals of the token
    /// * `total_supply`: [u64] - The total supply of the token
    /// * `asset_id`: [Option<AssetId>] - The asset ID of the token
    ///
    /// ### Reverts
    ///
    /// * If the contract is already initialized
    /// * If the asset ID is not provided in collateral mode
    #[storage(read, write)]
    fn initialize(
        owner: b256,
        mailbox_address: b256,
        mode: WarpRouteTokenMode,
        hook: b256,
        //Token
        token_name: String,
        token_symbol: String,
        decimals: u8,
        total_supply: u64,
        asset_id: Option<AssetId>,
    ) {
        require(
            _owner() == State::Uninitialized,
            WarpRouteError::AlreadyInitialized,
        );

        let owner_id = Identity::Address(Address::from(owner));
        initialize_ownership(owner_id);
        storage.mailbox.write(ContractId::from(mailbox_address));
        storage.default_hook.write(ContractId::from(hook));
        storage.token_mode.write(mode);

        //When creating a single new asset on Fuel, we recommend using the SubId::zero()
        let sub_id = SubId::zero();

        let asset_id = match mode {
            WarpRouteTokenMode::BRIDGED => {
                // Derive asset_id based on contract_id and sub_id for bridged mode
                AssetId::new(ContractId::this(), sub_id)
            }
            WarpRouteTokenMode::COLLATERAL => {
                // Require asset_id as input in collateral mode
                require(
                    asset_id
                        .is_some(),
                    WarpRouteError::AssetIdRequiredForCollateral,
                );
                asset_id.unwrap()
            }
        };
        storage.sub_id.write(sub_id);
        storage.asset_id.write(asset_id);

        _set_name(storage.name, asset_id, token_name);
        _set_symbol(storage.symbol, asset_id, token_symbol);
        _set_decimals(storage.decimals, asset_id, decimals);
        storage.total_supply.insert(asset_id, total_supply);
        storage.cumulative_supply.insert(asset_id, 0);
    }

    /// Transfers tokens to a remote domain
    ///
    /// ### Arguments
    ///
    /// * `destination_domain`: [u32] - The domain to transfer the tokens to
    /// * `recipient`: [b256] - The address of the recipient
    /// * `amount`: [u64] - The amount of tokens to transfer
    ///
    /// ### Reverts
    ///
    /// * If the contract is paused
    /// * If reentrancy is detected
    /// * If the amount provided is greater than amount sent
    /// * If the asset ID of the asset being transferred is not the same as the asset ID set on the contract
    /// * If any external call fails
    #[payable]
    #[storage(read, write)]
    fn transfer_remote(destination_domain: u32, recipient: b256, amount: u64) {
        reentrancy_guard();
        require_not_paused();

        require(msg_amount() >= amount, WarpRouteError::InsufficientFunds);

        let asset = storage.asset_id.read();
        require(msg_asset_id() == asset, WarpRouteError::PaymentError);

        // Depending on the mode, handle the token transfer
        match storage.token_mode.read() {
            WarpRouteTokenMode::BRIDGED => {
                //Burn has checks inside along with decreasing total supply
                _burn(storage.total_supply, storage.sub_id.read(), amount);
            }
            WarpRouteTokenMode::COLLATERAL => {
                //TODO: should not be transfered to the contract in the future
                transfer(Identity::ContractId(ContractId::this()), asset, amount);
            }
        }

        let mailbox = abi(Mailbox, b256::from(storage.mailbox.read()));
        let hook_contract = storage.default_hook.read();

        let asset_metadata = _get_metadata_of_asset(asset);
        let message_body = _build_token_metadata_bytes(recipient, amount, asset_metadata);

        //Dispatch the message to the destination domain
        let message_id = mailbox.dispatch {
            coins: this_balance(AssetId::base()),
            asset_id: b256::from(AssetId::base()),
            gas: this_balance(AssetId::base()),
        }(
            destination_domain,
            recipient,
            message_body,
            Bytes::new(), // TODO: check if metadata is required
            hook_contract,
        );

        log(TransferRemoteEvent {
            destination_domain,
            hook_contract,
            message_id,
        });
    }

    /// Handles a transfer from a remote domain
    ///
    /// ### Arguments
    ///
    /// * `id`: [b256] - The ID of the message
    /// * `origin`: [u32] - The domain of the origin
    /// * `sender`: [b256] - The address of the sender
    /// * `message_body`: [bytes] - The message body
    ///
    /// ### Reverts
    ///
    /// * If the contract is paused
    /// * If the message has already been delivered
    /// * If the cumulative supply exceeds the maximum supply
    #[storage(read, write)]
    fn handle_message(id: b256, _origin: u32, _sender: b256, message_body: Bytes) {
        require_not_paused();
        require(
            !storage
                .delivered_messages
                .get(id)
                .try_read()
                .unwrap_or(false),
            WarpRouteError::MessageAlreadyDelivered,
        );

        storage.delivered_messages.insert(id, true);

        let (recipient, amount, token_metadata) = _extract_asset_data_from_body(message_body);
        let recipient_identity = Identity::Address(Address::from(recipient));
        let asset = storage.asset_id.read();

        match storage.token_mode.read() {
            WarpRouteTokenMode::BRIDGED => {
                let cumulative_supply = storage.cumulative_supply.get(asset).read();

                require(
                    cumulative_supply + amount <= MAX_SUPPLY,
                    WarpRouteError::MaxMinted,
                );
                storage
                    .cumulative_supply
                    .insert(asset, cumulative_supply + amount);
                let _ = _mint(
                    storage
                        .total_assets,
                    storage
                        .total_supply,
                    recipient_identity,
                    storage
                        .sub_id
                        .read(),
                    amount,
                );
            }
            WarpRouteTokenMode::COLLATERAL => {
                transfer(recipient_identity, asset, amount);
            }
        }

        // Log the successful message handling
        log(HandleMessageEvent {
            recipient,
            amount,
            token_metadata,
        });
    }

    /// Gets the token mode of the WarpRoute contract
    ///
    /// ### Returns
    ///
    /// * [WarpRouteTokenMode] - The token mode
    #[storage(read)]
    fn get_token_mode() -> WarpRouteTokenMode {
        storage.token_mode.read()
    }

    /// Gets the token metadata of the WarpRoute contract
    ///
    /// ### Returns
    ///
    /// * [TokenMetadata] - The token metadata
    #[storage(read)]
    fn get_token_info() -> TokenMetadata {
        let asset = storage.asset_id.read();
        let res = _get_metadata_of_asset(asset);
        res
    }

    /// Gets the mailbox contract ID that the WarpRoute contract is using for transfers
    ///
    /// ### Returns
    /// 
    /// * [b256] - The mailbox contract ID
    #[storage(read)]
    fn get_mailbox() -> b256 {
        storage.mailbox.read().into()
    }

    /// Gets the total number of coins ever minted for an asset.
    ///
    /// ### Returns
    ///
    /// * [u64] - The total number of coins ever minted for an asset.
    #[storage(read)]
    fn get_cumulative_supply() -> u64 {
        let asset = storage.asset_id.read();
        storage.cumulative_supply.get(asset).try_read().unwrap_or(0)
    }

    /// Gets the post dispatch hook contract ID that the WarpRoute contract is using
    ///
    /// ### Returns
    ///
    /// * [b256] - The post dispatch hook contract ID
    #[storage(read)]
    fn get_hook() -> b256 {
        storage.default_hook.read().into()
    }

    /// Sets the mailbox contract ID that the WarpRoute contract is using for transfers
    ///
    /// ### Arguments
    ///
    /// * `mailbox_address`: [b256] - The mailbox contract ID
    ///
    /// ### Reverts
    ///
    /// * If the caller is not the owner
    /// * If the mailbox address is zero
    #[storage(write)]
    fn set_mailbox(mailbox_address: b256) {
        only_owner();
        require(!mailbox_address.is_zero(), WarpRouteError::InvalidAddress);
        storage.mailbox.write(ContractId::from(mailbox_address));
    }

    /// Sets the post dispatch hook contract ID that the WarpRoute contract is using
    ///
    /// ### Arguments
    ///
    /// * `hook`: [b256] - The post dispatch hook contract ID
    ///
    /// ### Reverts
    ///
    /// * If the caller is not the owner
    /// * If the hook address is zero
    #[storage(write)]
    fn set_hook(hook: b256) {
        only_owner();
        require(!hook.is_zero(), WarpRouteError::InvalidAddress);
        storage.default_hook.write(ContractId::from(hook));
    }

    /// Checks if a message has been delivered
    ///
    /// ### Arguments
    ///
    /// * `message_id`: [b256] - The ID of the message
    ///
    /// ### Returns
    ///
    /// * [bool] - Whether the message has been delivered
    #[storage(read)]
    fn is_message_delivered(message_id: b256) -> bool {
        storage.delivered_messages.get(message_id).try_read().unwrap_or(false)
    }

    // TODO: must be removed after unit and E2E testing 
    #[storage(read, write)]
    fn mint_tokens(recipient: Address, amount: u64) {
        let recipient_identity = Identity::Address(recipient);
        let asset = storage.asset_id.read();
        let cumulative_supply = storage.cumulative_supply.get(asset).read();

        require(
            cumulative_supply + amount <= MAX_SUPPLY,
            WarpRouteError::MaxMinted,
        );
        storage
            .cumulative_supply
            .insert(asset, cumulative_supply + amount);

        let _ = _mint(
            storage
                .total_assets,
            storage
                .total_supply,
            recipient_identity,
            storage
                .sub_id
                .read(),
            amount,
        );
    }
}

// ---------------  Pausable and Ownable  ---------------

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
        initialize_ownership(new_owner);
    }

    #[storage(read, write)]
    fn renounce_ownership() {
        renounce_ownership();
    }
}

// ---------------  Internal Functions  ---------------

#[storage(read)]
fn _get_metadata_of_asset(asset: AssetId) -> TokenMetadata {
    TokenMetadata {
        name: _name(storage.name, asset).unwrap_or(String::new()),
        symbol: _symbol(storage.symbol, asset).unwrap_or(String::new()),
        decimals: _decimals(storage.decimals, asset).unwrap_or(0),
        total_supply: storage.total_supply.get(asset).read(),
        asset_id: storage.asset_id.read(),
        sub_id: storage.sub_id.read(),
    }
}

fn _build_token_metadata_bytes(recipient: b256, amount: u64, metadata: TokenMetadata) -> Bytes {
    let mut buffer = Buffer::new();

    buffer = recipient.abi_encode(buffer);
    buffer = amount.abi_encode(buffer);
    buffer = metadata.decimals.abi_encode(buffer);
    buffer = metadata.total_supply.abi_encode(buffer);

    let bytes = Bytes::from(buffer.as_raw_slice());
    bytes
}

#[storage(read)]
fn _extract_asset_data_from_body(body: Bytes) -> (b256, u64, TokenMetadata) {
    let mut buffer_reader = BufferReader::from_parts(body.ptr(), body.len());

    let recipient = buffer_reader.read::<b256>();
    let amount = buffer_reader.read::<u64>();
    let decimals = buffer_reader.read::<u8>();
    let total_supply = buffer_reader.read::<u64>();

    let asset_id = storage.asset_id.read();
    let sub_id = storage.sub_id.read();

    let metadata = TokenMetadata {
        name: _name(storage.name, asset_id).unwrap_or(String::new()),
        symbol: _symbol(storage.symbol, asset_id).unwrap_or(String::new()),
        decimals,
        total_supply,
        asset_id,
        sub_id,
    };
    (recipient, amount, metadata)
}
