use ink_storage::traits::{
    PackedLayout,
    SpreadLayout,
};
use openbrush::{
    contracts::{
        ownable::OwnableError,
        psp34::Id,
    },
    storage::Mapping,
    traits::{
        AccountId,
        Balance,
        String,
    },
};
use scale::{
    Decode,
    Encode,
};

pub const STORAGE_KEY: u32 = openbrush::storage_unique_key!(Data);

#[derive(Default, Debug)]
#[openbrush::upgradeable_storage(STORAGE_KEY)]
pub struct Data {
    pub registered_contracts: Mapping<AccountId, RegisteredCollection>,
    pub items: Mapping<(AccountId, Id), Item>,
    pub fee: u16,
    pub max_fee: u16,
    pub market_fee_recipient: AccountId,
}

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum MarketplaceError {
    /// Caller is not a marketplace owner.
    OwnableError(OwnableError),
    /// Caller is not an NFT owner.
    NotOwner,
    /// A NFT item is not found in a contract.
    ItemNotFound,
    /// A NFT item is not listed for sale
    ItemNotListedForSale,
    /// NFT contract is not registered to the marketplace.
    NotRegisteredContract,
    /// Value send to buy method is invalid
    BadBuyValue,
    /// Fee transfer to the marketplace failed.
    TransferToMarketplaceFailed,
    /// Royalty transfer to the marketplace failed.
    TransferToOwnerFailed,
    ///
    TransferToAuthorFailed,
    ContractAlreadyRegistered,
    FeeToHigh,
}

#[derive(Encode, Decode, SpreadLayout, PackedLayout, Default, Debug)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout)
)]
pub struct RegisteredCollection {
    pub royalty_receiver: AccountId,
    pub metadata: String,
    pub royalty: u16,
}

#[derive(Encode, Decode, SpreadLayout, PackedLayout, Default, Debug)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout)
)]
pub struct Item {
    pub owner: AccountId,
    pub price: Balance,
}

impl From<OwnableError> for MarketplaceError {
    fn from(error: OwnableError) -> Self {
        MarketplaceError::OwnableError(error)
    }
}
