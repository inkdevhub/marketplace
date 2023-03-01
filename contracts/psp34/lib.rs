#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[openbrush::contract]
pub mod test_psp34 {
    // imports from openbrush
    use openbrush::{
        contracts::psp34::extensions::{
            burnable::*,
            metadata::*,
            mintable::*,
        },
        traits::{
            Storage,
            String,
        },
    };

    #[ink(storage)]
    #[derive(Default, Storage)]
    pub struct Contract {
        #[storage_field]
        psp34: psp34::Data,
        #[storage_field]
        metadata: metadata::Data,
    }

    // Section contains default implementation without any modifications
    impl PSP34 for Contract {}
    impl PSP34Burnable for Contract {}
    impl PSP34Mintable for Contract {}
    impl PSP34Metadata for Contract {}

    impl Contract {
        #[ink(constructor)]
        pub fn new() -> Self {
            let mut instance = Self::default();
            instance
                ._mint_to(instance.env().caller(), Id::U8(1))
                .expect("Can mint");
            let collection_id = instance.collection_id();
            instance._set_attribute(
                collection_id.clone(),
                String::from("name"),
                String::from("MyPSP34"),
            );
            instance._set_attribute(collection_id, String::from("symbol"), String::from("MPSP"));
            instance
        }
    }
}
