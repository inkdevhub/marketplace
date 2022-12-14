#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[openbrush::contract]
pub mod marketplace {
    use ink_storage::traits::SpreadAllocate;
    use openbrush::{
        contracts::{ownable::*, psp34::PSP34Error},
        traits::Storage,
    };
    use pallet_marketplace::{impls::marketplace::*, traits::marketplace::*};

    // MarketplaceContract contract storage
    #[ink(storage)]
    #[derive(Default, SpreadAllocate, Storage)]
    pub struct MarketplaceContract {
        #[storage_field]
        ownable: ownable::Data,
        #[storage_field]
        marketplace: types::Data,
    }

    /// Event emitted when a NFT contract registration occurs.
    #[ink(event)]
    pub struct ContractRegistered {
        #[ink(topic)]
        contract_address: AccountId,
        #[ink(topic)]
        user_address: AccountId,
    }

    pub type Result<T> = core::result::Result<T, PSP34Error>;

    impl MarketplaceContract {
        #[ink(constructor)]
        pub fn new() -> Self {
            ink_lang::codegen::initialize_contract(|instance: &mut MarketplaceContract| {
                // TODO do initialization
                instance.marketplace.fee = 100;
            })
        }
    }

    impl MarketplaceSale for MarketplaceContract {}
}

// ***************************** Tests *******************************
#[cfg(test)]
mod tests {
    use crate::marketplace::MarketplaceContract;
    use pallet_marketplace::traits::marketplace::*;
    use ink_lang as ink;

    #[ink::test]
    fn new_works() {
        let marketplace = init_contract();
        assert_eq!(marketplace.get_marketplace_fee(), 100);
    }

    fn init_contract() -> MarketplaceContract {
        MarketplaceContract::new()
    }
}
