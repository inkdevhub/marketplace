#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[openbrush::contract]
pub mod marketplace {
    use ink::{
        codegen::{
            EmitEvent,
            Env,
        },
        env::DefaultEnvironment,
        EnvAccess,
    };
    use openbrush::{
        contracts::{
            ownable::*,
            psp34::Id,
            reentrancy_guard::*,
        },
        traits::Storage,
    };
    use pallet_marketplace::{
        impls::marketplace::{
            marketplace_sale::MarketplaceSaleEvents,
            *,
        },
        traits::marketplace::*,
    };

    // MarketplaceContract contract storage
    #[ink(storage)]
    #[derive(Default, Storage)]
    pub struct MarketplaceContract {
        #[storage_field]
        ownable: ownable::Data,
        #[storage_field]
        guard: reentrancy_guard::Data,
        #[storage_field]
        marketplace: types::Data,
    }

    /// Event emitted when token is listed or unlisted
    #[ink(event)]
    pub struct TokenListed {
        #[ink(topic)]
        contract: AccountId,
        #[ink(topic)]
        id: Id,
        #[ink(topic)]
        price: Option<Balance>,
    }

    /// Event emitted when a token is bought
    #[ink(event)]
    pub struct TokenBought {
        #[ink(topic)]
        contract: AccountId,
        #[ink(topic)]
        id: Id,
        #[ink(topic)]
        price: Balance,
    }

    /// Event emitted when a NFT contract is registered to the marketplace.
    #[ink(event)]
    pub struct CollectionRegistered {
        #[ink(topic)]
        contract: AccountId,
    }

    impl MarketplaceContract {
        #[ink(constructor)]
        pub fn new(market_fee_recipient: AccountId) -> Self {
            let mut instance = Self::default();
            instance.marketplace.fee = 100; // 1%
            instance.marketplace.max_fee = 1000; // 10%
            instance.marketplace.market_fee_recipient = Option::Some(market_fee_recipient);

            let caller = instance.env().caller();
            instance._init_with_owner(caller);
            instance
        }
    }

    impl MarketplaceSaleEvents for MarketplaceContract {
        fn emit_token_listed_event(
            &self,
            contract: AccountId,
            token_id: Id,
            price: Option<Balance>,
        ) {
            <EnvAccess<'_, DefaultEnvironment> as EmitEvent<MarketplaceContract>>::emit_event::<
                TokenListed,
            >(
                self.env(),
                TokenListed {
                    contract,
                    id: token_id,
                    price,
                },
            );
        }

        fn emit_token_bought_event(&self, contract: AccountId, token_id: Id, price: Balance) {
            <EnvAccess<'_, DefaultEnvironment> as EmitEvent<MarketplaceContract>>::emit_event::<
                TokenBought,
            >(
                self.env(),
                TokenBought {
                    contract,
                    id: token_id,
                    price,
                },
            );
        }

        fn emit_collection_registered_event(&self, contract: AccountId) {
            <EnvAccess<'_, DefaultEnvironment> as EmitEvent<MarketplaceContract>>::emit_event::<
                CollectionRegistered,
            >(self.env(), CollectionRegistered { contract })
        }
    }

    impl MarketplaceSale for MarketplaceContract {}

    // ***************************** Tests *******************************
    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::marketplace::MarketplaceContract;
        use ink::env::test;
        use openbrush::{
            contracts::psp34::Id,
            traits::String,
        };
        use pallet_marketplace::impls::marketplace::types::{
            MarketplaceError,
            NftContractType,
        };

        #[ink::test]
        fn new_works() {
            let marketplace = init_contract();
            assert_eq!(marketplace.get_marketplace_fee(), 100);
            assert_eq!(marketplace.get_max_fee(), 1000);
            assert_eq!(marketplace.get_fee_recipient(), fee_recipient());
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
        fn set_marketplace_fee_fails_if_fee_too_high() {
            let mut marketplace = init_contract();

            assert_eq!(
                marketplace.set_marketplace_fee(1001),
                Err(MarketplaceError::FeeTooHigh)
            );
            assert!(marketplace.set_marketplace_fee(1000).is_ok());
        }

        #[ink::test]
        fn set_fee_recipient_works() {
            let mut marketplace = init_contract();
            let accounts = default_accounts();

            assert!(marketplace.set_fee_recipient(accounts.bob).is_ok());
            assert_eq!(marketplace.get_fee_recipient(), accounts.bob);
        }

        #[ink::test]
        fn set_fee_recipient_fails_if_not_owner() {
            let mut marketplace = init_contract();
            let accounts = default_accounts();
            set_sender(accounts.bob);

            assert_eq!(
                marketplace.set_fee_recipient(accounts.bob),
                Err(MarketplaceError::OwnableError(
                    OwnableError::CallerIsNotOwner
                ))
            );
        }

        #[ink::test]
        fn buy_fails_if_unlisted_token() {
            let mut marketplace = init_contract();

            assert_eq!(
                marketplace.buy(contract_address(), Id::U128(1)),
                Err(MarketplaceError::ItemNotListedForSale)
            );
        }

        #[ink::test]
        fn register_contract_works() {
            let mut marketplace = init_contract();
            let ipfs = String::from("ipfs");

            assert!(marketplace
                .register(contract_address(), fee_recipient(), 999, ipfs.clone())
                .is_ok());
            let contract = marketplace
                .get_registered_collection(contract_address())
                .unwrap();
            assert_eq!(contract.royalty_receiver, fee_recipient());
            assert_eq!(contract.royalty, 999);
            assert_eq!(contract.marketplace_ipfs, ipfs);
            assert_eq!(1, ink::env::test::recorded_events().count());
        }

        #[ink::test]
        fn register_fails_if_fee_too_high() {
            let mut marketplace = init_contract();
            let ipfs = String::from("ipfs");

            assert_eq!(
                marketplace.register(contract_address(), fee_recipient(), 1001, ipfs.clone()),
                Err(MarketplaceError::FeeTooHigh)
            );
            assert!(marketplace
                .register(contract_address(), fee_recipient(), 999, ipfs)
                .is_ok());
        }

        #[ink::test]
        fn register_fails_if_contract_already_registered() {
            let mut marketplace = init_contract();
            let ipfs = String::from("ipfs");

            assert!(marketplace
                .register(contract_address(), fee_recipient(), 999, ipfs.clone())
                .is_ok());
            assert_eq!(
                marketplace.register(contract_address(), fee_recipient(), 999, ipfs),
                Err(MarketplaceError::ContractAlreadyRegistered)
            );
        }

        #[ink::test]
        fn set_nft_contract_hash_works() {
            let mut marketplace = init_contract();
            let hash = Hash::try_from([1; 32]).unwrap();
            let hash2 = Hash::try_from([2; 32]).unwrap();

            assert!(marketplace
                .set_nft_contract_hash(NftContractType::Rmrk, hash)
                .is_ok());
            assert_eq!(marketplace.nft_contract_hash(NftContractType::Rmrk), hash);

            // Check also if owner is able to update hash.
            assert!(marketplace
                .set_nft_contract_hash(NftContractType::Rmrk, hash2)
                .is_ok());
            assert_eq!(marketplace.nft_contract_hash(NftContractType::Rmrk), hash2);
        }

        #[ink::test]
        fn set_nft_contract_fails_if_not_owner() {
            let mut marketplace = init_contract();
            let hash = Hash::try_from([1; 32]).unwrap();
            let accounts = default_accounts();
            set_sender(accounts.bob);

            assert_eq!(
                marketplace.set_nft_contract_hash(NftContractType::Rmrk, hash),
                Err(MarketplaceError::OwnableError(
                    OwnableError::CallerIsNotOwner
                ))
            );
        }

        #[ink::test]
        fn factory_fails_if_no_hash() {
            let mut marketplace = init_contract();
            let accounts = default_accounts();

            assert_eq!(
                marketplace.factory(
                    String::from("ipfs"),
                    accounts.alice,
                    100,
                    String::from("name"),
                    String::from("symbol"),
                    String::from("base_uri"),
                    0,
                    0,
                    NftContractType::Psp34
                ),
                Err(MarketplaceError::NftContractHashNotSet)
            );
        }

        fn init_contract() -> MarketplaceContract {
            MarketplaceContract::new(fee_recipient())
        }

        fn default_accounts() -> test::DefaultAccounts<ink::env::DefaultEnvironment> {
            test::default_accounts::<Environment>()
        }

        fn set_sender(sender: AccountId) {
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(sender);
        }

        fn fee_recipient() -> AccountId {
            AccountId::from([0x1; 32])
        }

        fn contract_address() -> AccountId {
            AccountId::from([0x2; 32])
        }
    }

    /// end-to-end (E2E) or integration tests for marketplace.
    ///
    /// When running these you need to make sure that you:
    /// - Compile the tests with the `e2e-tests` feature flag enabled (`--features e2e-tests`)
    /// - Are running a Substrate node which contains `pallet-contracts` in the background
    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// A helper function used for calling contract messages.
        use ink_e2e::build_message;

        /// The End-to-End test `Result` type.
        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        /// We test that we can upload and instantiate the contract using its default constructor.
        #[ink_e2e::test]
        async fn init_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let constructor = MarketplaceRef::new(fee_recipient());

            // When
            let contract_account_id = client
                .instantiate("marketplace", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            // Then
            let marketplace_fee = build_message::<MarketplaceRef>(contract_account_id.clone())
                .call(|marketplace| marketplace.get_marketplace_fee());
            let marketplace_fee_result = client.call_dry_run(&ink_e2e::alice(), &owner, 0, None).await;
            assert!(matches!(marketplace_fee_result.return_value(), 100));

            Ok(())
        }

        fn fee_recipient() -> AccountId {
            AccountId::from([0x1; 32])
        }
    }
}
