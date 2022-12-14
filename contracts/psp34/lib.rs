#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]
        
#[openbrush::contract]
pub mod test_psp34 {
    // imports from ink!
	use ink_prelude::string::String;
	use ink_storage::traits::SpreadAllocate;
    
    // imports from openbrush
	use openbrush::traits::Storage;
	use openbrush::contracts::psp34::extensions::burnable::*;
	use openbrush::contracts::psp34::extensions::mintable::*;
	use openbrush::contracts::psp34::extensions::enumerable::*;
	use openbrush::contracts::psp34::extensions::metadata::*;
    
    #[ink(storage)]
    #[derive(Default, SpreadAllocate, Storage)]
    pub struct Contract {
    	#[storage_field]
		psp34: psp34::Data<Balances>,
		#[storage_field]
		metadata: metadata::Data,
    }
    
    // Section contains default implementation without any modifications
	impl PSP34 for Contract {}
	impl PSP34Burnable for Contract {}
	impl PSP34Mintable for Contract {}
	impl PSP34Enumerable for Contract {}
	impl PSP34Metadata for Contract {}
    
    impl Contract {
        #[ink(constructor)]
        pub fn new() -> Self {
            ink_lang::codegen::initialize_contract(|_instance: &mut Contract|{
				_instance._mint_to(_instance.env().caller(), Id::U8(1)).expect("Can mint");
				let collection_id = _instance.collection_id();
				_instance._set_attribute(collection_id.clone(), String::from("name").into_bytes(), String::from("TestPSP34").into_bytes());
				_instance._set_attribute(collection_id, String::from("symbol").into_bytes(), String::from("TETS").into_bytes());
			})
        }
    }
}