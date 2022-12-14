use ink_prelude::vec::Vec;
use openbrush::contracts::ownable::OwnableError;
use openbrush::contracts::psp34::Id;
use openbrush::storage::Mapping;
use openbrush::traits::{AccountId, Balance, String};
use scale::{Decode, Encode};

pub const STORAGE_KEY: u32 = openbrush::storage_unique_key!(Data);

#[derive(Default, Debug)]
#[openbrush::upgradeable_storage(STORAGE_KEY)]
pub struct Data {
    pub registered_contracts: Vec<AccountId>,
    pub items: Mapping<(AccountId, Id), Item>,
    pub fee: u16,
}

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum MarketplaceError {
    OwnableError(OwnableError),
    SomethingIsWrong,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct Item {
    pub owner: AccountId,
    pub contract: AccountId,
    pub price: Balance,
}

impl From<OwnableError> for MarketplaceError {
    fn from(error: OwnableError) -> Self {
        MarketplaceError::OwnableError(error)
    }
}
