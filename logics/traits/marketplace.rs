use crate::impls::marketplace::types::MarketplaceError;
use ink_env::Hash;
use openbrush::{
    contracts::psp34::Id,
    traits::{
        AccountId,
        Balance,
        String,
    },
};

#[openbrush::trait_definition]
pub trait MarketplaceSale {
    /// Add NFT contract to the marketplace.
    #[ink(message)]
    fn factory(&mut self, hash: Hash, ipfs: String) -> Result<(), MarketplaceError>;

    /// Create NFT item sale on the marketplace.
    #[ink(message)]
    fn list(
        &mut self,
        contract_address: AccountId,
        token_id: Id,
        price: Balance,
    ) -> Result<(), MarketplaceError>;

    /// Removes NFT from the marketplace sale.
    fn unlist(&mut self, contract_address: AccountId, token_id: Id)
        -> Result<(), MarketplaceError>;

    /// Buy NFT item from the marketplace.
    #[ink(message, payable)]
    fn buy(&mut self, contract_address: AccountId, token_id: Id) -> Result<(), MarketplaceError>;

    /// Registers NFT contract to the marketplace.
    #[ink(message)]
    fn register(
        &mut self,
        contract_address: AccountId,
        royalty_receiver: AccountId,
        royalty: u16,
    ) -> Result<(), MarketplaceError>;

    /// Sets the marketplace fee.
    #[ink(message)]
    fn set_marketplace_fee(&mut self, fee: u16) -> Result<(), MarketplaceError>;

    /// Gets the marketplace fee.
    #[ink(message)]
    fn get_marketplace_fee(&self) -> u16;

    /// Gets max fee that can be applied to an item price.
    #[ink(message)]
    fn get_max_fee(&self) -> u16;

    /// Checks if NFT token is listed on the marketplace.
    #[ink(message)]
    fn get_price(&self, contract_address: AccountId, token_id: Id) -> Option<Balance>;

    #[ink(message)]
    fn set_contract_metadata(&mut self, ipfs: String) -> Result<(), MarketplaceError>;

    #[ink(message)]
    fn get_fee_recipient(&self) -> AccountId;

    #[ink(message)]
    fn set_fee_recipient(&mut self, fee_recipient: AccountId) -> Result<(), MarketplaceError>;
}
