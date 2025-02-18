library;

abi AggregationHook {
    /// Initializes the AggregationHook contract.
    ///
    /// ### Arguments
    ///
    /// * `owner`: [Identity] - The owner of the contract.
    /// * `hooks`: [Vec<ContractId>] - The hooks to initialize with.
    ///
    /// ### Reverts
    ///
    /// * If the contract is already initialized.
    #[storage(write)]
    fn initialize(owner: Identity, hooks: Vec<ContractId>);

    /// Gets the hooks.
    ///
    /// ### Returns
    ///
    /// * [Vec<ContractId>] - The hooks.
    ///
    /// ### Reverts
    ///
    /// * If the contract is not initialized.
    #[storage(read)]
    fn get_hooks() -> Vec<ContractId>;
}
