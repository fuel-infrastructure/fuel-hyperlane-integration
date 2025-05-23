use crate::helper::*;
use alloy::{
    network::{Ethereum, EthereumWallet},
    primitives::{address, Address as AlloyAddress, Bytes as AlloyBytes, FixedBytes, U256},
    providers::ProviderBuilder,
    sol,
    transports::BoxTransport,
};
use alloy_provider::{
    fillers::{
        BlobGasFiller, ChainIdFiller, FillProvider, GasFiller, JoinFill, NonceFiller, WalletFiller,
    },
    Identity, Provider, RootProvider,
};
use alloy_rpc_types::{BlockNumberOrTag, Filter};
use fuels::{
    accounts::wallet::WalletUnlocked,
    macros::abigen,
    programs::calls::{CallParameters, Execution},
    types::{
        bech32::Bech32ContractId, transaction_builders::VariableOutputPolicy, AssetId, Bits256,
        Bytes, ContractId,
    },
};
use futures_util::StreamExt;
use rand::{thread_rng, Rng};
use sepolia_warp_route_collateral::SepoliaWarpRouteCollateral::SepoliaWarpRouteCollateralInstance;
use sepolia_warp_route_synthetic::SepoliaWarpRouteSynthetic::SepoliaWarpRouteSyntheticInstance;
use std::env;
use SepoliaMailbox::SepoliaMailboxInstance;
use SepoliaRecipient::SepoliaRecipientInstance;

abigen!(
    Contract(
        name = "Mailbox",
        abi = "contracts/mailbox/out/debug/mailbox-abi.json",
    ),
    Contract(
        name = "PostDispatch",
        abi = "contracts/mocks/mock-post-dispatch/out/debug/mock-post-dispatch-abi.json",
    ),
    Contract(
        name = "MerkleTreeHook",
        abi = "contracts/hooks/merkle-tree-hook/out/debug/merkle-tree-hook-abi.json",
    ),
    Contract(
        name = "ValidatorAnnounce",
        abi = "contracts/validator-announce/out/debug/validator-announce-abi.json",
    ),
    Contract(
        name = "GasOracle",
        abi = "contracts/gas-oracle/out/debug/gas-oracle-abi.json",
    ),
    Contract(
        name = "GasPaymaster",
        abi = "contracts/hooks/gas-paymaster/out/debug/gas-paymaster-abi.json",
    ),
    Contract(
        name = "TestRecipient",
        abi = "contracts/test/msg-recipient-test/out/debug/msg-recipient-test-abi.json",
    ),
    Contract(
        name = "WarpRoute",
        abi = "contracts/warp-route/out/debug/warp-route-abi.json",
    ),
    Contract(
        name = "MessageIdMultisigISM",
        abi = "contracts/ism/multisig/message-id-multisig-ism/out/debug/message-id-multisig-ism-abi.json",
    ),
    Contract(
        name = "MerkleRootMultisigISM",
        abi = "contracts/ism/multisig/merkle-root-multisig-ism/out/debug/merkle-root-multisig-ism-abi.json",
    ),
);

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    SepoliaRecipient,
    "evm-abis/Recipient.json"
);

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    SepoliaMailbox,
    "evm-abis/Mailbox.json"
);

mod sepolia_warp_route_synthetic {
    use alloy::sol;

    sol!(
        #[allow(missing_docs)]
        #[sol(rpc)]
        SepoliaWarpRouteSynthetic,
        "evm-abis/HypERC20.json",
    );
}
mod sepolia_warp_route_collateral {
    use alloy::sol;

    sol!(
        #[allow(missing_docs)]
        #[sol(rpc)]
        SepoliaWarpRouteCollateral,
        "evm-abis/HypERC20Collateral.json",
    );
}

#[allow(clippy::type_complexity, dead_code)]
pub struct SepoliaContracts {
    pub mailbox: SepoliaMailboxInstance<
        BoxTransport,
        FillProvider<
            JoinFill<
                JoinFill<
                    Identity,
                    JoinFill<
                        GasFiller,
                        JoinFill<BlobGasFiller, JoinFill<NonceFiller, ChainIdFiller>>,
                    >,
                >,
                WalletFiller<EthereumWallet>,
            >,
            RootProvider<BoxTransport>,
            BoxTransport,
            Ethereum,
        >,
    >,
    pub recipient: SepoliaRecipientInstance<
        BoxTransport,
        FillProvider<
            JoinFill<
                JoinFill<
                    Identity,
                    JoinFill<
                        GasFiller,
                        JoinFill<BlobGasFiller, JoinFill<NonceFiller, ChainIdFiller>>,
                    >,
                >,
                WalletFiller<EthereumWallet>,
            >,
            RootProvider<BoxTransport>,
            BoxTransport,
            Ethereum,
        >,
    >,
    pub warp_route_collateral: SepoliaWarpRouteCollateralInstance<
        BoxTransport,
        FillProvider<
            JoinFill<
                JoinFill<
                    Identity,
                    JoinFill<
                        GasFiller,
                        JoinFill<BlobGasFiller, JoinFill<NonceFiller, ChainIdFiller>>,
                    >,
                >,
                WalletFiller<EthereumWallet>,
            >,
            RootProvider<BoxTransport>,
            BoxTransport,
            Ethereum,
        >,
    >,
    pub warp_route_synthetic: SepoliaWarpRouteSyntheticInstance<
        BoxTransport,
        FillProvider<
            JoinFill<
                JoinFill<
                    Identity,
                    JoinFill<
                        GasFiller,
                        JoinFill<BlobGasFiller, JoinFill<NonceFiller, ChainIdFiller>>,
                    >,
                >,
                WalletFiller<EthereumWallet>,
            >,
            RootProvider<BoxTransport>,
            BoxTransport,
            Ethereum,
        >,
    >,
    pub merkle_tree_hook: AlloyAddress,
    pub message_id_multisig_ism: AlloyAddress,
    pub merkle_root_multisig_ism: AlloyAddress,
    pub test_ism: AlloyAddress,
}

#[allow(dead_code)]
pub struct FuelContracts {
    pub mailbox: Mailbox<WalletUnlocked>,
    pub ism: ContractId,
    pub merkle_tree_hook: ContractId,
    pub validator_announce: ContractId,
    pub igp: GasPaymaster<WalletUnlocked>,
    pub gas_oracle: ContractId,
    pub test_recipient: TestRecipient<WalletUnlocked>,
    pub aggregation_ism: ContractId,
    pub domain_routing_ism: ContractId,
    pub fallback_domain_routing_ism: ContractId,
    pub message_id_multisig_ism_1: ContractId,
    pub merkle_root_multisig_ism_1: ContractId,
    pub message_id_multisig_ism_3: ContractId,
    pub merkle_root_multisig_ism_3: ContractId,
    pub test_ism: ContractId,
    pub warp_route_collateral: WarpRoute<WalletUnlocked>,
    pub warp_route_synthetic: WarpRoute<WalletUnlocked>,
    pub recipient: ContractId,
}

pub struct Contracts {
    pub fuel: FuelContracts,
    pub sepolia: SepoliaContracts,
}

pub type EvmProvider = FillProvider<
    JoinFill<
        JoinFill<
            Identity,
            JoinFill<GasFiller, JoinFill<BlobGasFiller, JoinFill<NonceFiller, ChainIdFiller>>>,
        >,
        WalletFiller<EthereumWallet>,
    >,
    RootProvider<BoxTransport>,
    BoxTransport,
    Ethereum,
>;

#[allow(dead_code)]
pub enum DispatchType {
    WithNoHook,
    WithIGPHook,
    WithMerkleTreeHook,
    TokenSend,
}

impl Contracts {
    pub async fn fuel_send_dispatch(&self, dispatch_type: DispatchType) -> (String, String) {
        let recipient_address = self.sepolia.recipient.address().to_vec();
        let mut address_array = [0u8; 32];
        address_array[12..].copy_from_slice(&recipient_address);

        let rnd_number = thread_rng().gen_range(0..10000);
        let body_text = format!("Hello from Fuel! {}", rnd_number);

        let hook = match dispatch_type {
            DispatchType::WithIGPHook => self.fuel.igp.contract_id(),
            DispatchType::WithMerkleTreeHook => &Bech32ContractId::from(self.fuel.merkle_tree_hook),
            DispatchType::WithNoHook => &Bech32ContractId::default(),
            DispatchType::TokenSend => &Bech32ContractId::default(),
        };
        let body = hex::encode(body_text).into_bytes();

        let res = match dispatch_type {
            DispatchType::TokenSend => {
                self.fuel
                    .mailbox
                    .methods()
                    .dispatch(
                        84532,
                        Bits256(address_array),
                        Bytes(body),
                        Bytes(vec![0]),
                        hook,
                    )
                    .call_params(CallParameters::new(223526, get_native_asset(), 223526))
                    .unwrap()
                    .with_contracts(&[&self.fuel.igp])
                    .determine_missing_contracts(Some(10))
                    .await
                    .unwrap()
                    .call()
                    .await
            }
            _ => {
                self.fuel
                    .mailbox
                    .methods()
                    .dispatch(
                        84532,
                        Bits256(address_array),
                        Bytes(body),
                        Bytes(vec![0]),
                        hook,
                    )
                    .with_contracts(&[&self.fuel.igp])
                    .with_variable_output_policy(VariableOutputPolicy::EstimateMinimum)
                    .determine_missing_contracts(Some(10))
                    .await
                    .unwrap()
                    .call()
                    .await
            }
        };

        match res {
            Ok(res) => {
                let message_id = format!("0x{}", hex::encode(res.value.0));
                let tx_id = res.tx_id.unwrap();
                println!(
                    "Dispatch from Fuel successful at: 0x{:?}",
                    res.tx_id.unwrap()
                );
                (message_id, format!("0x{:?}", tx_id))
            }
            Err(e) => {
                println!("Dispatch error: {:?}", e);
                panic!();
            }
        }
    }

    pub async fn sepolia_send_dispatch(
        &self,
        dispatch_type: DispatchType,
    ) -> (FixedBytes<32>, FixedBytes<32>) {
        let recipient_address = ContractId::from(self.fuel.test_recipient.contract_id()).to_vec();
        let parsed_address: FixedBytes<32> = FixedBytes::from_slice(recipient_address.as_slice());
        let rnd_number = thread_rng().gen_range(0..10000);
        let body_text = format!("Hello from sepolia! {}", rnd_number);
        let body = AlloyBytes::copy_from_slice(body_text.as_bytes());
        let metadata = AlloyBytes::copy_from_slice("".as_bytes());

        let res = match dispatch_type {
            DispatchType::WithNoHook => {
                self.sepolia
                    .mailbox
                    .dispatch_2(get_fuel_chain_id(), parsed_address, body)
                    .value(U256::from(1))
                    .send()
                    .await
                    .unwrap()
                    .watch()
                    .await
            }
            DispatchType::WithMerkleTreeHook => {
                self.sepolia
                    .mailbox
                    .dispatch_0(
                        get_fuel_chain_id(),
                        parsed_address,
                        body,
                        metadata,
                        self.sepolia.merkle_tree_hook,
                    )
                    .value(U256::from(1))
                    .send()
                    .await
                    .unwrap()
                    .watch()
                    .await
            }
            _ => panic!("Invalid dispatch type"),
        };

        match res {
            Ok(tx_id) => {
                println!("Dispatch from Sepolia successful at: {:?}", tx_id);
                let message_id = self
                    .sepolia
                    .mailbox
                    .latestDispatchedId()
                    .call()
                    .await
                    .unwrap()
                    ._0;

                (message_id, tx_id)
            }
            Err(e) => {
                println!("Dispatch error: {:?}", e);
                panic!();
            }
        }
    }

    pub async fn set_fuel_mailbox_ism_to_message_id_threshold_1(&self) {
        let res = self
            .fuel
            .mailbox
            .methods()
            .set_default_ism(self.fuel.message_id_multisig_ism_1)
            .call()
            .await
            .unwrap();

        println!(
            "Mailbox ISM set to MessageIdMultisig 1/x at: {:?}",
            res.tx_id.unwrap()
        );
    }

    pub async fn set_fuel_mailbox_ism_to_merkle_root_threshold_1(&self) {
        let res = self
            .fuel
            .mailbox
            .methods()
            .set_default_ism(self.fuel.merkle_root_multisig_ism_1)
            .call()
            .await
            .unwrap();

        println!(
            "Mailbox ISM set to MerkleRootMultisig 1/x at: {:?}",
            res.tx_id.unwrap()
        );
    }

    pub async fn set_fuel_mailbox_ism_to_message_id_threshold_3(&self) {
        let res = self
            .fuel
            .mailbox
            .methods()
            .set_default_ism(self.fuel.message_id_multisig_ism_3)
            .call()
            .await
            .unwrap();

        println!(
            "Mailbox ISM set to MessageIdMultisig 3/x at: {:?}",
            res.tx_id.unwrap()
        );
    }

    pub async fn set_fuel_mailbox_ism_to_merkle_root_threshold_3(&self) {
        let res = self
            .fuel
            .mailbox
            .methods()
            .set_default_ism(self.fuel.merkle_root_multisig_ism_3)
            .call()
            .await
            .unwrap();

        println!(
            "Mailbox ISM set to MerkleRootMultisig 3/x at: {:?}",
            res.tx_id.unwrap()
        );
    }

    pub async fn set_fuel_mailbox_ism_to_test_ism(&self) {
        let res = self
            .fuel
            .mailbox
            .methods()
            .set_default_ism(self.fuel.test_ism)
            .call()
            .await
            .unwrap();

        println!("Mailbox ISM set to Test ISM at: {:?}", res.tx_id.unwrap());
    }

    pub async fn set_fuel_ism_to_aggregation(&self) {
        let res = self
            .fuel
            .test_recipient
            .methods()
            .set_ism(self.fuel.aggregation_ism)
            .call()
            .await
            .unwrap();

        println!("ISM set to Aggregation at: {:?}", res.tx_id.unwrap());
    }

    pub async fn set_fuel_ism_to_domain_routing(&self) {
        let res = self
            .fuel
            .test_recipient
            .methods()
            .set_ism(self.fuel.domain_routing_ism)
            .call()
            .await
            .unwrap();

        println!("ISM set to Domain Routing at: {:?}", res.tx_id.unwrap());
    }

    pub async fn set_fuel_ism_to_fallback_domain_routing(&self) {
        let res = self
            .fuel
            .test_recipient
            .methods()
            .set_ism(self.fuel.fallback_domain_routing_ism)
            .call()
            .await
            .unwrap();

        println!(
            "ISM set to Fallback Domain Routing at: {:?}",
            res.tx_id.unwrap()
        );
    }

    pub async fn set_fuel_ism_to_test_ism(&self) {
        let res = self
            .fuel
            .test_recipient
            .methods()
            .set_ism(self.fuel.test_ism)
            .call()
            .await
            .unwrap();

        println!("ISM set to Test ISM at: {:?}", res.tx_id.unwrap());
    }

    pub async fn set_sepolia_ism_to_message_id_multisig(&self) {
        let res = self
            .sepolia
            .recipient
            .setInterchainSecurityModule(self.sepolia.message_id_multisig_ism)
            .send()
            .await
            .unwrap()
            .watch()
            .await
            .unwrap();

        println!("ISM set to Message ID Multisig at: {:?}", res);
    }

    pub async fn set_sepolia_ism_to_merkle_root_multisig(&self) {
        let res = self
            .sepolia
            .recipient
            .setInterchainSecurityModule(self.sepolia.merkle_root_multisig_ism)
            .send()
            .await
            .unwrap()
            .watch()
            .await
            .unwrap();

        println!("ISM set to Merkle Root Multisig at: {:?}", res);
    }

    pub async fn set_sepolia_ism_to_test_ism(&self) {
        let res = self
            .sepolia
            .recipient
            .setInterchainSecurityModule(self.sepolia.test_ism)
            .send()
            .await
            .unwrap()
            .watch()
            .await
            .unwrap();

        println!("ISM set to Test ISM at: {:?}", res);
    }

    pub async fn fuel_quote_dispatch(&self) -> u64 {
        let gas_payment_quote = self
            .fuel
            .igp
            .methods()
            .quote_gas_payment(get_basesepolia_chain_id(), 5000)
            .with_contract_ids(&[self.fuel.gas_oracle.into()])
            .call()
            .await
            .map_err(|e| println!("Fuel quote gas payment error: {:?}", e))
            .unwrap();

        gas_payment_quote.value
    }

    pub async fn claim_gas_payment(&self) {
        let res = self
            .fuel
            .igp
            .methods()
            .claim(None)
            .with_variable_output_policy(VariableOutputPolicy::Exactly(5))
            .call()
            .await;
        match res {
            Ok(res) => {
                println!(
                  "Claim gas payment from Fuel successful: https://app-testnet.fuel.network/tx/0x{:?}",
                  res.tx_id.unwrap()
              );
            }
            Err(e) => {
                println!("Claim gas payment from Fuel error: {:?}", e);
            }
        }
    }

    pub async fn fuel_get_minted_asset_id(&self) -> AssetId {
        self.fuel
            .warp_route_synthetic
            .methods()
            .get_token_info()
            .call()
            .await
            .unwrap()
            .value
            .asset_id
    }

    /// Fuel (ETH) -> Sepolia (USDC)
    pub async fn fuel_transfer_remote_collateral(&self, amount: u64) {
        let recipient_address = self.sepolia.recipient.address().to_vec();
        let mut address_array = [0u8; 32];
        address_array[12..].copy_from_slice(&recipient_address);

        let remote_wr = self.sepolia.warp_route_collateral.address().to_vec();
        let mut remote_wr_array = [0u8; 32];
        remote_wr_array[12..].copy_from_slice(&remote_wr);

        let _add_router_res = self
            .fuel
            .warp_route_collateral
            .methods()
            .enroll_remote_router(get_basesepolia_chain_id(), Bits256(remote_wr_array))
            .call()
            .await
            .unwrap();

        let _adjust_decimals_res = self
            .fuel
            .warp_route_collateral
            .methods()
            .set_remote_router_decimals(Bits256(remote_wr_array), 6)
            .call()
            .await
            .unwrap();

        let _set_hook_res = self
            .fuel
            .warp_route_collateral
            .methods()
            .set_hook(self.fuel.igp.contract_id())
            .call()
            .await
            .unwrap();

        let _res = self
            .sepolia
            .warp_route_collateral
            .setInterchainSecurityModule(self.sepolia.test_ism)
            .send()
            .await
            .unwrap()
            .watch()
            .await;

        let router_address = self.fuel.warp_route_collateral.contract_id().hash();
        let parsed_router_address: FixedBytes<32> =
            FixedBytes::from_slice(router_address.as_slice());

        let _res = self
            .sepolia
            .warp_route_collateral
            .enrollRemoteRouter(get_fuel_chain_id(), parsed_router_address)
            .send()
            .await
            .unwrap()
            .watch()
            .await;

        let quote = self.fuel_quote_dispatch().await;

        let res = self
            .fuel
            .warp_route_collateral
            .methods()
            .transfer_remote(
                get_basesepolia_chain_id(),
                Bits256(address_array),
                amount,
                None,
                None,
            )
            .with_variable_output_policy(VariableOutputPolicy::Exactly(5))
            .call_params(CallParameters::new(
                amount + quote,
                get_native_asset(),
                3_000_000,
            ))
            .unwrap()
            .determine_missing_contracts(Some(8))
            .await
            .unwrap()
            .call()
            .await;

        match res {
            Ok(res) => {
                println!(
                    "Transfer from fuel successful: https://app-testnet.fuel.network/tx/0x{:?}",
                    res.tx_id.unwrap()
                );
            }
            Err(e) => {
                println!("Transfer remote (ETH) from fuel error: {:?}", e);
            }
        }
    }

    /// Fuel (FST) -> Sepolia (FST)
    pub async fn fuel_transfer_remote_synthetic(&self, wallet: WalletUnlocked, amount: u64) {
        let recipient_address = self.sepolia.recipient.address().to_vec();
        let mut address_array = [0u8; 32];
        address_array[12..].copy_from_slice(&recipient_address);

        let remote_wr_address = self.sepolia.warp_route_synthetic.address().to_vec();
        let mut remote_wr_address_array = [0u8; 32];
        remote_wr_address_array[12..].copy_from_slice(&remote_wr_address);

        let _add_router_res = self
            .fuel
            .warp_route_synthetic
            .methods()
            .enroll_remote_router(get_basesepolia_chain_id(), Bits256(remote_wr_address_array))
            .with_variable_output_policy(VariableOutputPolicy::Exactly(5))
            .call()
            .await
            .unwrap();

        let _adjust_decimals_res = self
            .fuel
            .warp_route_synthetic
            .methods()
            .set_remote_router_decimals(Bits256(remote_wr_address_array), 9)
            .call()
            .await
            .unwrap();

        let _set_ism = self
            .fuel
            .warp_route_synthetic
            .methods()
            .set_ism(self.fuel.test_ism)
            .call()
            .await
            .unwrap();

        let _set_hook_res = self
            .fuel
            .warp_route_synthetic
            .methods()
            .set_hook(self.fuel.igp.contract_id())
            .call()
            .await
            .unwrap();

        let _res = self
            .sepolia
            .warp_route_synthetic
            .setInterchainSecurityModule(self.sepolia.test_ism)
            .send()
            .await
            .unwrap()
            .watch()
            .await;

        let router_address = self.fuel.warp_route_synthetic.contract_id().hash();
        let parsed_router_address: FixedBytes<32> =
            FixedBytes::from_slice(router_address.as_slice());

        let _res = self
            .sepolia
            .warp_route_synthetic
            .enrollRemoteRouter(get_fuel_chain_id(), parsed_router_address)
            .send()
            .await
            .unwrap()
            .watch()
            .await;

        let asset_id = self.fuel_get_minted_asset_id().await;
        let quote = self.fuel_quote_dispatch().await;

        send_token_to_contract(wallet, self.fuel.warp_route_synthetic.contract_id(), amount).await;

        let res = self
            .fuel
            .warp_route_synthetic
            .methods()
            .transfer_remote(
                get_basesepolia_chain_id(),
                Bits256(address_array),
                amount,
                None,
                None,
            )
            .with_variable_output_policy(VariableOutputPolicy::Exactly(5))
            .call_params(CallParameters::new(quote, asset_id, 223_526))
            .unwrap()
            .determine_missing_contracts(Some(5))
            .await
            .unwrap()
            .call()
            .await;

        match res {
            Ok(res) => {
                println!(
                    "Transfer from fuel successful: https://app-testnet.fuel.network/tx/0x{:?}",
                    res.tx_id.unwrap()
                );
            }
            Err(e) => {
                println!("Transfer remote synthetic from fuel error: {:?}", e);
            }
        }
    }

    /// Sepolia (FST) -> Fuel (FST)
    pub async fn sepolia_transfer_remote_synthetic(&self, amount: u64) -> FixedBytes<32> {
        let recipient_address = ContractId::from(self.fuel.test_recipient.contract_id()).to_vec();
        let parsed_address: FixedBytes<32> = FixedBytes::from_slice(recipient_address.as_slice());

        let router_address = self.fuel.warp_route_synthetic.contract_id().hash();
        let parsed_router_address: FixedBytes<32> =
            FixedBytes::from_slice(router_address.as_slice());

        let _ = self
            .sepolia
            .warp_route_synthetic
            .enrollRemoteRouter(get_fuel_chain_id(), parsed_router_address)
            .send()
            .await
            .unwrap()
            .watch()
            .await;

        let remote_wr_address = self.sepolia.warp_route_synthetic.address().to_vec();
        let mut remote_wr_address_array = [0u8; 32];
        remote_wr_address_array[12..].copy_from_slice(&remote_wr_address);

        let _set_router = self
            .fuel
            .warp_route_synthetic
            .methods()
            .enroll_remote_router(get_basesepolia_chain_id(), Bits256(remote_wr_address_array))
            .call()
            .await
            .unwrap();

        let _adjust_decimals_res = self
            .fuel
            .warp_route_synthetic
            .methods()
            .set_remote_router_decimals(Bits256(remote_wr_address_array), 6)
            .call()
            .await
            .unwrap();

        let res = self
            .sepolia
            .warp_route_synthetic
            .transferRemote_1(get_fuel_chain_id(), parsed_address, U256::from(amount))
            .value(U256::from(1)) // qoute dispatch result
            .send()
            .await
            .unwrap()
            .watch()
            .await;

        match res {
            Ok(res) => {
                println!(
                    "Transfer remote synthetic from sepolia successful: {:?}",
                    res
                );

                self.sepolia
                    .mailbox
                    .latestDispatchedId()
                    .call()
                    .await
                    .unwrap()
                    ._0
            }
            Err(e) => {
                println!("Dispatch error: {:?}", e);
                panic!();
            }
        }
    }

    /// Sepolia (USDC) -> Fuel (ETH)
    pub async fn sepolia_transfer_remote_collateral(&self, amount: u64) -> FixedBytes<32> {
        let recipient_address = ContractId::from(self.fuel.test_recipient.contract_id()).to_vec();
        let parsed_address: FixedBytes<32> = FixedBytes::from_slice(recipient_address.as_slice());

        let router_address = self.fuel.warp_route_collateral.contract_id().hash();
        let parsed_router_address: FixedBytes<32> =
            FixedBytes::from_slice(router_address.as_slice());

        let _res = self
            .sepolia
            .warp_route_collateral
            .enrollRemoteRouter(get_fuel_chain_id(), parsed_router_address)
            .send()
            .await
            .unwrap()
            .watch()
            .await;

        let remote_wr_address = self.sepolia.warp_route_synthetic.address().to_vec();
        let mut remote_wr_address_array = [0u8; 32];
        remote_wr_address_array[12..].copy_from_slice(&remote_wr_address);

        let _ = self
            .fuel
            .warp_route_collateral
            .methods()
            .set_remote_router_decimals(Bits256(remote_wr_address_array), 9)
            .call()
            .await
            .unwrap();

        let res = self
            .sepolia
            .warp_route_collateral
            .transferRemote_1(get_fuel_chain_id(), parsed_address, U256::from(amount))
            .value(U256::from(1)) // qoute dispatch result
            .send()
            .await
            .unwrap()
            .watch()
            .await;

        match res {
            Ok(res) => {
                println!("Transfer remote collateral successful: {:?}", res);

                self.sepolia
                    .mailbox
                    .latestDispatchedId()
                    .call()
                    .await
                    .unwrap()
                    ._0
            }
            Err(e) => {
                println!("Dispatch error: {:?}", e);
                panic!();
            }
        }
    }

    pub async fn monitor_fuel_for_delivery(&self, message_id: FixedBytes<32>) {
        println!("Monitoring Fuel for delivery");
        let message_id = Bits256(message_id.0);

        loop {
            let delivered_res = self
                .fuel
                .mailbox
                .methods()
                .delivered(message_id)
                .simulate(Execution::StateReadOnly)
                .await
                .unwrap();

            println!("Is message delivered: {:?} \n", delivered_res.value);

            if delivered_res.value {
                break;
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
        }
    }

    pub async fn monitor_sepolia_for_delivery(&self) -> FixedBytes<32> {
        let ws_rpc_url = env::var("SEPOLIA_WS_RPC_URL").expect("SEPOLIA_WS_RPC_URL must be set");
        let provider = ProviderBuilder::new()
            .on_builtin(ws_rpc_url.as_str())
            .await
            .unwrap();

        // let mailbox_address = address!("c2E0b1526E677EA0a856Ec6F50E708502F7fefa9");
        let recipient_address = self.sepolia.recipient.address().to_owned();
        let filter = Filter::new()
            .address(recipient_address)
            .event("ReceivedMessage(uint32,bytes32,uint256,string)")
            .from_block(BlockNumberOrTag::Latest);

        let sub = provider.subscribe_logs(&filter).await.unwrap();
        let mut stream = sub.into_stream();

        let mut tx_id = FixedBytes::default();
        if let Some(log) = stream.next().await {
            tx_id = log.transaction_hash.unwrap();
            println!("Sepolia Mailbox received message at: {:?}\n", tx_id);
        }
        tx_id
    }

    pub async fn monitor_sepolio_for_asset_delivery(&self, is_synthetic: bool) {
        let ws_rpc_url = env::var("SEPOLIA_WS_RPC_URL").expect("SEPOLIA_WS_RPC_URL must be set");
        let provider = ProviderBuilder::new()
            .on_builtin(ws_rpc_url.as_str())
            .await
            .unwrap();

        let wr_address = if is_synthetic {
            self.sepolia.warp_route_synthetic.address().to_owned()
        } else {
            self.sepolia.warp_route_collateral.address().to_owned()
        };

        let filter = Filter::new()
            .address(wr_address)
            .event("ReceivedTransferRemote(uint32,bytes32,uint256)")
            .from_block(BlockNumberOrTag::Latest);

        let sub = provider.subscribe_logs(&filter).await.unwrap();
        let mut stream = sub.into_stream();

        if let Some(log) = stream.next().await {
            if let Some(tx_hash) = log.transaction_hash {
                println!(
                    "Recieve from sepolia successful: https://sepolia.etherscan.io/tx/{:?}",
                    tx_hash
                );
            } else {
                println!(
                    "Transaction hash not available in the log.Full log: {:?}",
                    log
                );
            }
        }
    }
}

pub async fn load_contracts(fuel_wallet: WalletUnlocked, evm_provider: EvmProvider) -> Contracts {
    // fuel contract addresses
    let mailbox_id = get_contract_id_from_json("fueltestnet", &["mailbox"]);

    let igp = get_contract_id_from_json("fueltestnet", &["interchainGasPaymaster"]);
    let ism = get_contract_id_from_json("fueltestnet", &["interchainSecurityModule"]);
    let merkle_tree_hook = get_contract_id_from_json("fueltestnet", &["merkleTreeHook"]);
    let validator_announce = get_contract_id_from_json("fueltestnet", &["validatorAnnounce"]);
    let gas_oracle = get_contract_id_from_json("fueltestnet", &["storageGasOracle"]);
    let warp_route_collateral = get_contract_id_from_json("fueltestnet", &["warpRouteNative"]);
    let warp_route_synthetic = get_contract_id_from_json("fueltestnet", &["warpRouteSynthetic"]);
    let recipient = get_contract_id_from_json("fueltestnet", &["recipient"]);
    let yaml_config = read_deployments_yaml();
    // Fuel instances
    let mailbox_instance_fuel = Mailbox::new(mailbox_id, fuel_wallet.clone());
    let igp_instance = GasPaymaster::new(igp, fuel_wallet.clone());
    let msg_recipient_instance =
        TestRecipient::new(yaml_config.test_recipient, fuel_wallet.clone());
    let warp_route_collateral_instance = WarpRoute::new(warp_route_collateral, fuel_wallet.clone());
    let warp_route_synthetic_instance = WarpRoute::new(warp_route_synthetic, fuel_wallet.clone());

    // Base Sepolia contract addresses
    let mailbox_instance_sepolia = SepoliaMailbox::new(
        address!("6966b0E55883d49BFB24539356a2f8A673E02039"),
        evm_provider.clone(),
    );
    let sepolia_recipient = SepoliaRecipient::new(
        address!("845b94653A03AF56fb38fBC673491eB0A009fE34"),
        evm_provider.clone(),
    );

    let warp_route_collateral_instance_sepolia = SepoliaWarpRouteCollateralInstance::new(
        address!("13F558Fbf6E98eDfA158E9b6bC447f356201E649"),
        evm_provider.clone(),
    );

    let warp_route_synthetic_instance_sepolia = SepoliaWarpRouteSyntheticInstance::new(
        address!("c10f47030fEB9E4111e6b591354dd0EB01C02919"),
        evm_provider.clone(),
    );

    Contracts {
        fuel: FuelContracts {
            mailbox: mailbox_instance_fuel,
            igp: igp_instance,
            ism,
            merkle_tree_hook,
            validator_announce,
            gas_oracle,
            aggregation_ism: yaml_config.aggregation_ism,
            domain_routing_ism: yaml_config.domain_routing_ism,
            fallback_domain_routing_ism: yaml_config.fallback_domain_routing_ism,
            message_id_multisig_ism_1: yaml_config.message_id_multisig_ism_1,
            merkle_root_multisig_ism_1: yaml_config.merkle_root_multisig_ism_1,
            message_id_multisig_ism_3: yaml_config.message_id_multisig_ism_3,
            merkle_root_multisig_ism_3: yaml_config.merkle_root_multisig_ism_3,
            test_recipient: msg_recipient_instance,
            test_ism: yaml_config.test_ism,
            warp_route_collateral: warp_route_collateral_instance,
            warp_route_synthetic: warp_route_synthetic_instance,
            recipient,
        },
        sepolia: SepoliaContracts {
            mailbox: mailbox_instance_sepolia,
            recipient: sepolia_recipient,
            merkle_tree_hook: address!("86fb9F1c124fB20ff130C41a79a432F770f67AFD"),
            message_id_multisig_ism: address!("5Fe883ad5BFe31942e2c383eb49e3c96eE053079"),
            merkle_root_multisig_ism: address!("8C94BA5A741a842eAa6eADd9CcA9d8B658D50024"),
            test_ism: address!("D873EFBBb00d08D08B2Af7A314f03658e199df98"),
            warp_route_collateral: warp_route_collateral_instance_sepolia,
            warp_route_synthetic: warp_route_synthetic_instance_sepolia,
        },
    }
}
