use crate::in_place::{InPlace, InPlaceCreate, InPlaceGet, InPlaceRead, InPlaceSet, InPlaceWrite};
use crate::util::AdvanceArray;
use crate::CruiserResult;
use solana_program::pubkey::Pubkey;

impl<'a> InPlace<'a> for Pubkey {
    type Access = &'a Pubkey;
    type AccessMut = &'a mut Pubkey;
}
impl<'a> InPlaceCreate<'a, ()> for Pubkey {
    fn create_with_arg(_data: &mut [u8], _arg: ()) -> CruiserResult {
        Ok(())
    }
}
impl<'a> InPlaceRead<'a, ()> for Pubkey {
    fn read_with_arg(mut data: &'a [u8], _arg: ()) -> CruiserResult<Self::Access> {
        let data: &[u8; 32] = data.try_advance_array()?;
        // Safe because Pubkey is transparent to [u8; 32]
        Ok(unsafe { &*data.as_ptr().cast::<Pubkey>() })
    }
}
impl<'a> InPlaceWrite<'a, ()> for Pubkey {
    fn write_with_arg(mut data: &'a mut [u8], _arg: ()) -> CruiserResult<Self::AccessMut> {
        let data: &mut [u8; 32] = data.try_advance_array()?;
        // Safe because Pubkey is transparent to [u8; 32]
        Ok(unsafe { &mut *data.as_mut_ptr().cast::<Pubkey>() })
    }
}
impl<'a> InPlaceGet<Pubkey> for <Pubkey as InPlace<'a>>::Access {
    fn get(&self) -> CruiserResult<Pubkey> {
        Ok(**self)
    }
}
impl<'a> InPlaceGet<Pubkey> for <Pubkey as InPlace<'a>>::AccessMut {
    fn get(&self) -> CruiserResult<Pubkey> {
        Ok(**self)
    }
}
impl<'a> InPlaceSet<Pubkey> for <Pubkey as InPlace<'a>>::AccessMut {
    fn set(&mut self, val: Pubkey) -> CruiserResult {
        **self = val;
        Ok(())
    }
}
