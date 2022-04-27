#![feature(const_trait_impl)]
#![feature(generic_associated_types)]
#![feature(const_mut_refs)]
// Solana uses rust 1.59, this does not support the new where clause location
#![allow(deprecated_where_clause_location)]

use cruiser::in_place::{
    get_properties, GetNum, InPlace, InPlaceCreate, InPlaceProperties, InPlacePropertiesList,
    InPlaceProperty, InPlaceRawDataAccess, InPlaceRawDataAccessMut, InPlaceRead, InPlaceWrite,
    SetNum,
};
use cruiser::on_chain_size::OnChainSize;
use cruiser::util::{Length, MappableRef, MappableRefMut, TryMappableRef, TryMappableRefMut};
use cruiser::{CruiserResult, Pubkey};
use std::error::Error;
use std::ops::{Deref, DerefMut};

pub struct TestData {
    pub value: u8,
    pub cool: [u16; 2],
    pub key: Pubkey,
}

impl const OnChainSize for TestData {
    const ON_CHAIN_SIZE: usize =
        u8::ON_CHAIN_SIZE + <[u16; 2]>::ON_CHAIN_SIZE + Pubkey::ON_CHAIN_SIZE;
}

impl InPlace for TestData {
    type Access<'a, A>
    where
        Self: 'a,
        A: 'a + MappableRef + TryMappableRef,
    = TestDataAccess<A>;
}

impl InPlaceProperties for TestData {
    type Properties = TestDataProperties;
}

impl InPlaceCreate for TestData {
    fn create_with_arg<A: DerefMut<Target = [u8]>>(mut data: A, arg: ()) -> CruiserResult {
        let mut data = &mut *data;
        <u8 as InPlaceCreate>::create_with_arg(
            cruiser::util::Advance::try_advance(&mut data, u8::ON_CHAIN_SIZE)?,
            arg,
        )?;
        <[u16; 2] as InPlaceCreate>::create_with_arg(
            cruiser::util::Advance::try_advance(&mut data, u8::ON_CHAIN_SIZE)?,
            arg,
        )?;
        <Pubkey as InPlaceCreate>::create_with_arg(
            cruiser::util::Advance::try_advance(&mut data, u8::ON_CHAIN_SIZE)?,
            arg,
        )?;
        Ok(())
    }
}

impl InPlaceRead for TestData {
    fn read_with_arg<'a, A>(data: A, _arg: ()) -> CruiserResult<Self::Access<'a, A>>
    where
        Self: 'a,
        A: 'a + Deref<Target = [u8]> + MappableRef + TryMappableRef,
    {
        Ok(TestDataAccess(data))
    }
}

impl InPlaceWrite for TestData {
    fn write_with_arg<'a, A>(data: A, _arg: ()) -> CruiserResult<Self::AccessMut<'a, A>>
    where
        Self: 'a,
        A: 'a
            + DerefMut<Target = [u8]>
            + MappableRef
            + TryMappableRef
            + MappableRefMut
            + TryMappableRefMut,
    {
        Ok(TestDataAccess(data))
    }
}

pub struct TestDataAccess<A>(A);

impl<A> const InPlaceRawDataAccess for TestDataAccess<A>
where
    A: ~const Deref<Target = [u8]>,
{
    fn get_raw_data(&self) -> &[u8] {
        &*self.0
    }
}

impl<A> const InPlaceRawDataAccessMut for TestDataAccess<A>
where
    A: ~const DerefMut<Target = [u8]>,
{
    fn get_raw_data_mut(&mut self) -> &mut [u8] {
        &mut *self.0
    }
}

#[derive(Copy, Clone, Debug)]
pub enum TestDataProperties {
    Value,
    Cool,
    Key,
}

impl const InPlacePropertiesList for TestDataProperties {
    fn index(self) -> usize {
        self as usize
    }

    fn offset(self) -> usize {
        match self {
            TestDataProperties::Value => 0,
            TestDataProperties::Cool => TestDataProperties::Value.offset() + u8::ON_CHAIN_SIZE,
            TestDataProperties::Key => {
                TestDataProperties::Cool.offset() + <[u16; 2] as OnChainSize>::ON_CHAIN_SIZE
            }
        }
    }

    fn size(self) -> Option<usize> {
        match self {
            TestDataProperties::Value => Some(u8::ON_CHAIN_SIZE),
            TestDataProperties::Cool => Some(<[u16; 2]>::ON_CHAIN_SIZE),
            TestDataProperties::Key => Some(Pubkey::ON_CHAIN_SIZE),
        }
    }
}

impl<A> const InPlaceProperty<0> for TestDataAccess<A> {
    type Property = u8;
}

impl<A> const InPlaceProperty<2> for TestDataAccess<A> {
    type Property = Pubkey;
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
    let (value, key) = get_properties!(&mut write_data, TestData { value, key })?;
    assert_eq!(value.get_num(), 2);
    assert_eq!(*key, Pubkey::new_from_array([1; 32]));
    Ok(())
}
