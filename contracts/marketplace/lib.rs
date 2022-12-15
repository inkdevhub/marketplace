#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[openbrush::contract]
pub mod marketplace {
    use ink_storage::traits::SpreadAllocate;
    use openbrush::{
        contracts::{
            ownable::*,
            psp34::PSP34Error,
        },
        traits::Storage,
    };
    use pallet_marketplace::{
        impls::marketplace::*,
        traits::marketplace::*,
    };

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
        pub fn new(market_fee_recepient: AccountId) -> Self {
            ink_lang::codegen::initialize_contract(|instance: &mut MarketplaceContract| {
                // TODO do initialization
                instance.marketplace.fee = 100;
                instance.marketplace.market_fee_recepient = market_fee_recepient;

                let caller = instance.env().caller();
                instance._init_with_owner(caller);
            })
        }
    }

    impl MarketplaceSale for MarketplaceContract {}

    // ***************************** Tests *******************************
    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::marketplace::MarketplaceContract;
        use ink_env::test;
        use ink_lang as ink;
        use pallet_marketplace::impls::marketplace::types::MarketplaceError;

        #[ink::test]
        fn new_works() {
            let marketplace = init_contract();
            assert_eq!(marketplace.get_marketplace_fee(), 100);
            assert_eq!(marketplace.get_fee_recepient(), fee_recipient());
        }

        #[ink::test]
        fn set_marketplace_fee_works() {
            let mut marketplace = init_contract();

            assert!(marketplace.set_marketplace_fee(120).is_ok());
            assert_eq!(marketplace.get_marketplace_fee(), 120);
        }

        #[ink::test]
        fn set_marketplace_fee_fails_if_not_owner() {
            let mut marketplace = init_contract();

            let accounts = default_accounts();
            set_sender(accounts.bob);
            assert_eq!(
                marketplace.set_marketplace_fee(120),
                Err(MarketplaceError::OwnableError(
                    OwnableError::CallerIsNotOwner
                ))
            );
        }

        #[ink::test]
        fn set_fee_recepient_works() {
            let mut marketplace = init_contract();
            let accounts = default_accounts();

            assert!(marketplace.set_fee_recepient(accounts.bob).is_ok());
            assert_eq!(marketplace.get_fee_recepient(), accounts.bob);
        }

        #[ink::test]
        fn set_fee_recepient_fails_if_not_owner() {
            let mut marketplace = init_contract();
            let accounts = default_accounts();
            set_sender(accounts.bob);

            assert_eq!(
                marketplace.set_fee_recepient(accounts.bob),
                Err(MarketplaceError::OwnableError(
                    OwnableError::CallerIsNotOwner
                ))
            );
        }

        fn init_contract() -> MarketplaceContract {
            MarketplaceContract::new(fee_recipient())
        }

        fn default_accounts() -> test::DefaultAccounts<ink_env::DefaultEnvironment> {
            test::default_accounts::<Environment>()
        }

        fn set_sender(sender: AccountId) {
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(sender);
        }

        fn fee_recipient() -> AccountId {
            AccountId::from([0x1; 32])
        }
    }
}
