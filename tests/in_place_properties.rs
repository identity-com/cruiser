#![feature(const_trait_impl)]
#![feature(generic_associated_types)]
#![feature(const_mut_refs)]
// Solana uses rust 1.59, this does not support the new where clause location
#![allow(deprecated_where_clause_location)]

use cruiser::in_place::{get_properties, GetNum, InPlace, InPlaceWrite, SetNum};
use cruiser::on_chain_size::OnChainSize;
use cruiser::util::Length;
use cruiser::Pubkey;
use std::error::Error;

#[derive(InPlace)]
pub struct TestData {
    pub value: u8,
    pub cool: [u16; 2],
    #[in_place(dynamic_size)]
    pub key: Pubkey,
}

impl const OnChainSize for TestData {
    const ON_CHAIN_SIZE: usize =
        u8::ON_CHAIN_SIZE + <[u16; 2]>::ON_CHAIN_SIZE + Pubkey::ON_CHAIN_SIZE;
}

#[test]
fn main_test() -> Result<(), Box<dyn Error>> {
    let mut data = [0u8; TestData::ON_CHAIN_SIZE];
    let mut write_data = TestData::write_with_arg(data.as_mut_slice(), ())?;
    let (mut value, mut key) = get_properties!(&mut write_data, TestData { value, key })?;
    assert_eq!(value.get_num(), 0);
    value.set_num(2);
    assert_eq!(*key, Pubkey::new_from_array([0; 32]));
    *key = Pubkey::new_from_array([1; 32]);
    drop((value, key));
    drop(write_data);
    let mut write_data = TestData::write_with_arg(data.as_mut_slice(), ())?;
    let (value, cool, key) = get_properties!(&mut write_data, TestData { value, cool, key })?;
    assert_eq!(value.get_num(), 2);
    assert_eq!(*key, Pubkey::new_from_array([1; 32]));
    cool.all()
        .for_each(|element| assert_eq!(element.unwrap().get_num(), 0));
    Ok(())
}
