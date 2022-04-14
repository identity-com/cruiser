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
    fn get_in_place(&self) -> Pubkey {
        **self
    }
}
impl<'a> InPlaceGet<Pubkey> for <Pubkey as InPlace<'a>>::AccessMut {
    fn get_in_place(&self) -> Pubkey {
        **self
    }
}
impl<'a> InPlaceSet<Pubkey> for <Pubkey as InPlace<'a>>::AccessMut {
    fn set_in_place(&mut self, val: Pubkey) {
        **self = val;
    }
}

#[cfg(test)]
mod test {
    use crate::in_place::{
        InPlaceGet, InPlaceSet, InPlaceUnitCreate, InPlaceUnitRead, InPlaceUnitWrite,
    };
    use rand::{thread_rng, Rng};
    use solana_program::pubkey::Pubkey;

    #[test]
    fn pubkey_test() {
        let rng = &mut thread_rng();
        for _ in 0..1024 {
            let value = Pubkey::new_from_array(rng.gen::<[u8; 32]>());
            let mut data = [0u8; 32];
            Pubkey::create(&mut data).expect("Could not create");
            let in_place = Pubkey::read(&data).expect("Could not read");
            assert_eq!(Pubkey::new_from_array([0; 32]), in_place.get_in_place());
            let mut in_place = Pubkey::write(&mut data).expect("Could not write");
            in_place.set_in_place(value);
            assert_eq!(value, in_place.get_in_place());
        }
    }
}
