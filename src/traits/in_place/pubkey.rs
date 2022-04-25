use crate::in_place::{InPlace, InPlaceCreate, InPlaceRead, InPlaceWrite};
use crate::{CruiserResult, GenericError};
use solana_program::pubkey::Pubkey;
use std::ops::{Deref, DerefMut};

impl InPlace for Pubkey {
    type Access<A> = PubkeyAccess<A>;
}
/// In-place accessor for [`Pubkey`]
#[derive(Debug)]
pub struct PubkeyAccess<A>(A);
impl<A> PubkeyAccess<A>
where
    A: Deref<Target = [u8]>,
{
    fn new(access: A) -> CruiserResult<Self> {
        if access.len() < 32 {
            Err(GenericError::NotEnoughData {
                needed: 32,
                remaining: access.len(),
            }
            .into())
        } else {
            Ok(Self(access))
        }
    }
}
impl<A> Deref for PubkeyAccess<A>
where
    A: Deref<Target = [u8]>,
{
    type Target = Pubkey;

    fn deref(&self) -> &Self::Target {
        debug_assert!(self.0.len() >= 32);
        unsafe { &*self.0.as_ptr().cast::<Pubkey>() }
    }
}
impl<A> DerefMut for PubkeyAccess<A>
where
    A: DerefMut<Target = [u8]>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        debug_assert!(self.0.len() >= 32);
        unsafe { &mut *self.0.as_mut_ptr().cast::<Pubkey>() }
    }
}
impl InPlaceCreate for Pubkey {
    fn create_with_arg<A>(_data: A, _arg: ()) -> CruiserResult
    where
        A: DerefMut<Target = [u8]>,
    {
        Ok(())
    }
}
impl<'a> InPlaceCreate<&'a Pubkey> for Pubkey {
    fn create_with_arg<A>(mut data: A, arg: &'a Pubkey) -> CruiserResult
    where
        A: DerefMut<Target = [u8]>,
    {
        data[..32].copy_from_slice(arg.as_ref());
        Ok(())
    }
}
impl InPlaceRead for Pubkey {
    fn read_with_arg<A>(data: A, _arg: ()) -> CruiserResult<Self::Access<A>>
    where
        A: Deref<Target = [u8]>,
    {
        PubkeyAccess::new(data)
    }
}
impl InPlaceWrite for Pubkey {
    fn write_with_arg<A>(data: A, _arg: ()) -> CruiserResult<Self::Access<A>>
    where
        A: DerefMut<Target = [u8]>,
    {
        PubkeyAccess::new(data)
    }
}

#[cfg(test)]
mod test {
    use crate::in_place::{InPlaceCreate, InPlaceRead, InPlaceWrite};
    use rand::{thread_rng, Rng};
    use solana_program::pubkey::Pubkey;

    #[test]
    fn pubkey_test() {
        let rng = &mut thread_rng();
        for _ in 0..1024 {
            let value = Pubkey::new_from_array(rng.gen::<[u8; 32]>());
            let mut data = [0u8; 32];
            Pubkey::create_with_arg(data.as_mut_slice(), ()).expect("Could not create");
            let in_place = Pubkey::read_with_arg(data.as_slice(), ()).expect("Could not read");
            assert_eq!(Pubkey::new_from_array([0; 32]), *in_place);
            let mut in_place =
                Pubkey::write_with_arg(data.as_mut_slice(), ()).expect("Could not write");
            *in_place = value;
            assert_eq!(value, *in_place);
        }
    }
}
