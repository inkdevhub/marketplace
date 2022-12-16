// Copyright (c) 2022 Astar Network
//
// Permission is hereby granted, free of charge, to any person obtaining
// a copy of this software and associated documentation files (the"Software"),
// to deal in the Software without restriction, including
// without limitation the rights to use, copy, modify, merge, publish,
// distribute, sublicense, and/or sell copies of the Software, and to
// permit persons to whom the Software is furnished to do so, subject to
// the following conditions:
//
// The above copyright notice and this permission notice shall be
// included in all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
// EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
// MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
// NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE
// LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
// OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION
// WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

use core::default;

use crate::{
    impls::marketplace::types::{
        Data,
        Item,
        MarketplaceError,
    },
    traits::marketplace::MarketplaceSale,
};
use ink_env::Hash;
use openbrush::{
    contracts::{
        ownable::*,
        psp34::*,
    },
    modifiers,
    traits::{
        AccountId,
        Balance,
        Storage,
        String,
    },
};

use super::types::RegisteredCollection;

pub trait Internal {
    /// Checks if contract caller is an token owner
    fn check_owner(
        &self,
        contract_address: AccountId,
        token_id: Id,
    ) -> Result<(), MarketplaceError>;

    fn check_value(
        &self,
        transfered_value: Balance,
        price: Balance,
    ) -> Result<(), MarketplaceError>;

    fn check_fee(&self, fee: u16, max_fee: u16) -> Result<(), MarketplaceError>;
}

impl<T> MarketplaceSale for T
where
    T: Storage<Data> + Storage<ownable::Data>,
{
    default fn factory(&mut self, hash: Hash, ipfs: String) -> Result<(), MarketplaceError> {
        Ok(())
    }

    default fn list(
        &mut self,
        contract_address: AccountId,
        token_id: Id,
        price: Balance,
    ) -> Result<(), MarketplaceError> {
        self.check_owner(contract_address, token_id.clone())?;
        self.data::<Data>().items.insert(
            &(contract_address, token_id),
            &Item {
                owner: Self::env().caller(),
                price,
            },
        );
        Ok(())
    }

    default fn unlist(
        &mut self,
        contract_address: AccountId,
        token_id: Id,
    ) -> Result<(), MarketplaceError> {
        self.check_owner(contract_address, token_id.clone())?;

        // TODO check what happens if key is not in collection.
        self.data::<Data>()
            .items
            .remove(&(contract_address, token_id));
        Ok(())
    }

    default fn buy(
        &mut self,
        contract_address: AccountId,
        token_id: Id,
    ) -> Result<(), MarketplaceError> {
        if let Some(item) = self.data::<Data>().items.get(&(contract_address, token_id)) {
            let value = Self::env().transferred_value();
            self.check_value(value, item.price)?;

            let marketplace_fee = value
                .checked_mul(self.data::<Data>().fee as u128)
                .unwrap_or_default()
                .checked_div(10_000)
                .unwrap_or_default();

            let collection = self
                .data::<Data>()
                .registered_contracts
                .get(&contract_address)
                .unwrap();
            let author_royalty = value
                .checked_mul(collection.royalty as u128)
                .unwrap_or_default()
                .checked_div(10_000)
                .unwrap_or_default();
            let seller_fee = value
                .checked_sub(marketplace_fee)
                .unwrap_or_default()
                .checked_sub(author_royalty)
                .unwrap_or_default();

            Self::env()
                .transfer(self.data::<Data>().market_fee_recipient, marketplace_fee)
                .map_err(|_| MarketplaceError::TransferToMarketplaceFailed)?;
            Self::env()
                .transfer(item.owner, seller_fee)
                .map_err(|_| MarketplaceError::TransferToOwnerFailed)?;
            Self::env()
                .transfer(collection.royalty_receiver, author_royalty)
                .map_err(|_| MarketplaceError::TransferToAuthorFailed)?;

            return Ok(())
        } else {
            return Err(MarketplaceError::ItemNotListedForSale)
        }
    }

    default fn register(
        &mut self,
        contract_address: AccountId,
        royalty_receiver: AccountId,
        royalty: u16,
    ) -> Result<(), MarketplaceError> {
        let max_fee = self.data::<Data>().max_fee;
        self.check_fee(royalty, max_fee)?;

        // TODO check owner

        if self
            .data::<Data>()
            .registered_contracts
            .get(&contract_address)
            .is_some()
        {
            return Err(MarketplaceError::ContractAlreadyRegistered)
        } else {
            self.data::<Data>().registered_contracts.insert(
                &contract_address,
                &RegisteredCollection {
                    royalty_receiver,
                    royalty,
                    metadata: String::from(""),
                },
            );

            return Ok(())
        }
    }

    #[modifiers(only_owner)]
    default fn set_marketplace_fee(&mut self, fee: u16) -> Result<(), MarketplaceError> {
        let max_fee = self.data::<Data>().max_fee;
        self.check_fee(fee, max_fee)?;
        self.data::<Data>().fee = fee;
        Ok(())
    }

    default fn get_marketplace_fee(&self) -> u16 {
        self.data::<Data>().fee
    }

    default fn get_max_fee(&self) -> u16 {
        self.data::<Data>().max_fee
    }

    default fn get_price(&self, contract_address: AccountId, token_id: Id) -> Option<Balance> {
        if let Some(item) = self.data::<Data>().items.get(&(contract_address, token_id)) {
            return Some(item.price)
        }

        None
    }

    default fn set_contract_metadata(&mut self, ipfs: String) -> Result<(), MarketplaceError> {
        Ok(())
    }

    default fn get_fee_recipient(&self) -> AccountId {
        self.data::<Data>().market_fee_recipient
    }

    #[modifiers(only_owner)]
    default fn set_fee_recipient(
        &mut self,
        fee_recipient: AccountId,
    ) -> Result<(), MarketplaceError> {
        self.data::<Data>().market_fee_recipient = fee_recipient;

        Ok(())
    }
}

impl<T> Internal for T
where
    T: Storage<Data>,
{
    default fn check_owner(
        &self,
        contract_address: AccountId,
        token_id: Id,
    ) -> Result<(), MarketplaceError> {
        if !self
            .data::<Data>()
            .registered_contracts
            .contains(&contract_address)
        {
            return Err(MarketplaceError::NotRegisteredContract)
        }

        let caller = Self::env().caller();
        if let Some(token_owner) = PSP34Ref::owner_of(&contract_address, token_id.clone()) {
            if caller != token_owner {
                return Err(MarketplaceError::NotOwner)
            }
        } else {
            return Err(MarketplaceError::ItemNotFound)
        }

        Ok(())
    }

    default fn check_value(
        &self,
        transfered_value: Balance,
        price: Balance,
    ) -> Result<(), MarketplaceError> {
        if transfered_value < price {
            return Err(MarketplaceError::BadBuyValue)
        }

        Ok(())
    }

    default fn check_fee(&self, fee: u16, max_fee: u16) -> Result<(), MarketplaceError> {
        if fee > max_fee {
            return Err(MarketplaceError::FeeToHigh)
        }

        Ok(())
    }
}
