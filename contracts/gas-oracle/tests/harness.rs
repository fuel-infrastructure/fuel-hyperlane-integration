use fuels::{
    prelude::*,
    types::{Bits256, Identity},
};

use test_utils::{funded_wallet_with_private_key, get_revert_reason};

abigen!(Contract(
    name = "GasOracle",
    abi = "contracts/gas-oracle/out/debug/gas-oracle-abi.json"
));

const NON_OWNER_PRIVATE_KEY: &str =
    "0xde97d8624a438121b86a1956544bd72ed68cd69f2c99555b08b1e8c51ffd511c";

async fn get_contract_instance() -> (GasOracle<WalletUnlocked>, ContractId) {
    // Launch a local network and deploy the contract
    let mut wallets = launch_custom_provider_and_get_wallets(
        WalletsConfig::new(
            Some(1),             /* Single wallet */
            Some(1),             /* Single coin (UTXO) */
            Some(1_000_000_000), /* Amount per coin */
        ),
        None,
        None,
    )
    .await
    .unwrap();

    let wallet = wallets.pop().unwrap();
    // Bits256(storage_gas_oracle_id.hash().into()),

    let expected_owner = Bits256(wallet.address().hash().into());
    let configurables = GasOracleConfigurables::default()
        .with_EXPECTED_OWNER(expected_owner)
        .unwrap();

    let id = Contract::load_from(
        "./out/debug/gas-oracle.bin",
        LoadConfiguration::default().with_configurables(configurables),
    )
    .unwrap()
    .deploy(&wallet, TxPolicies::default())
    .await
    .unwrap();

    let owner_identity = Identity::Address(wallet.address().into());

    let instance = GasOracle::new(id.clone(), wallet);

    instance
        .methods()
        .initialize_ownership(owner_identity)
        .call()
        .await
        .unwrap();

    (instance, id.into())
}

fn get_test_remote_gas_data_configs() -> Vec<RemoteGasDataConfig> {
    vec![
        RemoteGasDataConfig {
            domain: 11111,
            remote_gas_data: RemoteGasData {
                domain: 11111,
                token_exchange_rate: 22222_u64.into(),
                gas_price: 33333_u64.into(),
                token_decimals: 18u8,
            },
        },
        RemoteGasDataConfig {
            domain: 44444,
            remote_gas_data: RemoteGasData {
                domain: 44444,
                token_exchange_rate: 55555_u64.into(),
                gas_price: 66666_u64.into(),
                token_decimals: 9u8,
            },
        },
    ]
}

// ============ Set/Get Remote Gas Data Configs ============
#[tokio::test]
async fn test_set_remote_gas_data_configs_and_get_remote_gas_data() {
    let (oracle, _) = get_contract_instance().await;

    let configs = get_test_remote_gas_data_configs();

    oracle
        .methods()
        .set_remote_gas_data_configs(configs.clone())
        .call()
        .await
        .unwrap();

    // Ensure now `get_remote_gas_data` returns
    // the newly set values
    for config in configs {
        let remote_gas_data = oracle
            .methods()
            .get_remote_gas_data(config.domain)
            .simulate(Execution::StateReadOnly)
            .await
            .unwrap()
            .value;
        assert_eq!(remote_gas_data, config.remote_gas_data);
    }
}

// ============ Get Exchange Rate and Gas Price for Unknown Domain ============
#[tokio::test]
async fn test_exchange_rate_and_gas_price_unknown_domain() {
    let (oracle, _) = get_contract_instance().await;

    let RemoteGasData {
        domain: _,
        token_exchange_rate,
        gas_price,
        token_decimals,
    } = oracle
        .methods()
        .get_remote_gas_data(1234)
        .simulate(Execution::StateReadOnly)
        .await
        .unwrap()
        .value;
    assert_eq!(token_exchange_rate, 0);
    assert_eq!(gas_price, 0);
    assert_eq!(token_decimals, 9u8);
}

// ============ Set Remote Gas Data Configs Reverts if Not Owner ============
#[tokio::test]
async fn test_set_remote_gas_data_configs_reverts_if_not_owner() {
    let (oracle, _) = get_contract_instance().await;
    let binding = oracle.account();
    let non_owner_wallet = funded_wallet_with_private_key(&binding, NON_OWNER_PRIVATE_KEY);

    let non_owner_identity = Identity::Address(non_owner_wallet.await.address().into());

    oracle
        .methods()
        .transfer_ownership(non_owner_identity)
        .call()
        .await
        .unwrap();

    let configs = get_test_remote_gas_data_configs();
    let call = oracle
        .methods()
        .set_remote_gas_data_configs(configs)
        .call()
        .await;
    assert!(call.is_err());
    assert_eq!(get_revert_reason(call.err().unwrap()), "NotOwner");
}
