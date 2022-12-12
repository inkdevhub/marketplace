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

use crate::impls::marketplace::types::{Data, MarketplaceError};
pub use crate::traits::marketplace::MarketplaceSale;
use openbrush::traits::{AccountId, Balance, Storage};

impl<T> MarketplaceSale for T
where
    T: Storage<Data>,
{
    default fn add_nft_contract(
        &mut self,
        contract_address: AccountId,
    ) -> Result<(), MarketplaceError> {
        Ok(())
    }

    default fn add_market_data(
        &mut self,
        contract_address: AccountId,
        token_id: u64,
        price: Balance,
    ) -> Result<(), MarketplaceError> {
        Ok(())
    }

    default fn buy_item(
        &mut self,
        contract_address: AccountId,
        token_id: u64,
    ) -> Result<(), MarketplaceError> {
        Ok(())
    }
}
