module aegis_vault::escrow {
    use sui::coin::{Coin};
    use sui::sui::SUI;
    use sui::transfer;
    use sui::tx_context::{TxContext};

    public entry fun process_offline_transfer(
        payment: Coin<SUI>,       
        recipient: address,       
        ctx: &mut TxContext       
    ) {
        transfer::public_transfer(payment, recipient);
    }
}