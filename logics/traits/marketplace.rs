use crate::impls::marketplace::types::MarketplaceError;
use openbrush::traits::{AccountId, Balance};

#[openbrush::trait_definition]
pub trait MarketplaceSale {
    /// Add NFT contract to the marketplace.
    #[ink(message)]
    fn add_nft_contract(&mut self, contract_address: AccountId) -> Result<(), MarketplaceError>;

    /// Create NFT item sale on the marketplace.
    #[ink(message)]
    fn add_market_data(
        &mut self,
        contract_address: AccountId,
        token_id: u64,
        price: Balance,
    ) -> Result<(), MarketplaceError>;

    /// Buy NFT item from the marketplace.
    #[ink(message, payable)]
    fn buy_item(
        &mut self,
        contract_address: AccountId,
        token_id: u64,
    ) -> Result<(), MarketplaceError>;
}
