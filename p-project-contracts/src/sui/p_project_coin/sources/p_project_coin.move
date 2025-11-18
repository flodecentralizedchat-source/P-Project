// P-Project Coin Module for Sui
// 
// This module implements the P-Project token for the Sui blockchain.
// It follows the Sui coin standard and includes minting and burning capabilities.

module p_project::p_project_coin {
    use std::option;
    use sui::coin;
    use sui::transfer;
    use sui::tx_context::TxContext;

    /// The P-Project coin type
    struct PProject has drop {}

    /// Module initializer - creates the coin metadata
    fun init(witness: PProject, ctx: &mut TxContext) {
        let (treasury, metadata) = coin::create_currency(witness, 18, b"P-Project", b"P", b"", ctx);
        transfer::public_transfer(treasury, ctx.sender());
        transfer::public_transfer(metadata, ctx.sender());
    }

    /// Mint new P-Project tokens
    public entry fun mint(
        treasury: &mut coin::TreasuryCap<PProject>,
        amount: u64,
        recipient: address,
        ctx: &mut TxContext
    ) {
        let coins = coin::mint(treasury, amount, ctx);
        transfer::public_transfer(coins, recipient);
    }

    /// Burn P-Project tokens
    public entry fun burn(
        treasury: &mut coin::TreasuryCap<PProject>,
        coins: coin::Coin<PProject>
    ) {
        let amount = coin::value(&coins);
        coin::burn(treasury, coins);
        // In a real implementation, you might want to emit an event here
    }

    /// Transfer P-Project tokens
    public entry fun transfer(
        coins: coin::Coin<PProject>,
        recipient: address
    ) {
        transfer::public_transfer(coins, recipient);
    }

    /// Split P-Project tokens
    public entry fun split(
        coins: coin::Coin<PProject>,
        amount: u64,
        ctx: &mut TxContext
    ): (coin::Coin<PProject>, coin::Coin<PProject>) {
        coin::divide_and_keep(coins, amount, ctx)
    }

    /// Merge P-Project tokens
    public entry fun merge(
        coin1: &mut coin::Coin<PProject>,
        coin2: coin::Coin<PProject>
    ) {
        coin::merge(coin1, coin2);
    }

    /// Get the balance of P-Project tokens
    public entry fun balance(
        addr: address
    ): u64 {
        coin::balance<PProject>(addr)
    }

    /// Check if an address has any P-Project tokens
    public entry fun is_registered(
        addr: address
    ): bool {
        coin::is_registered<PProject>(addr)
    }

    #[test]
    public fun test_coin_operations() {
        // This is a placeholder for tests
        // In a real implementation, you would write actual tests here
    }
}