#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[openbrush::contract]
pub mod marketplace {
    use ink_storage::traits::SpreadAllocate;
    use openbrush::{
        contracts::{psp34::PSP34Error},
        traits::Storage,
    };
    use pallet_marketplace:: {
      traits::marketplace::*
    };

    // MarketplaceContract contract storage
    #[ink(storage)]
    #[derive(Default, SpreadAllocate, Storage)]
    pub struct MarketplaceContract {
        test: u32,
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
            Self { test: 0 }
        }
    }

    impl MarketplaceSale for MarketplaceContract {}
}
