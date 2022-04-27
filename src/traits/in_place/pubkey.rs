use crate::in_place::{InPlace, InPlaceCreate, InPlaceRead, InPlaceWrite};
use crate::util::{MappableRef, MappableRefMut, TryMappableRef, TryMappableRefMut};
use crate::{CruiserResult, GenericError};
use solana_program::pubkey::Pubkey;
use std::ops::{Deref, DerefMut};

impl InPlace for Pubkey {
    type Access<'a, A>
    where
        Self: 'a,
        A: 'a + MappableRef + TryMappableRef,
    = PubkeyAccess<'a, A>;
    type AccessMut<'a, A>
    where
        Self: 'a,
        A: 'a + MappableRef + TryMappableRef + MappableRefMut + TryMappableRefMut,
    = PubkeyAccessMut<'a, A>;
}

/// In-place accessor for [`Pubkey`]
#[derive(Debug)]
pub struct PubkeyAccess<'a, A>(A::Output<'a, [u8; 32]>)
where
    A: 'a + TryMappableRef;

impl<'a, A> PubkeyAccess<'a, A>
where
    A: Deref<Target = [u8]> + TryMappableRef,
{
    fn new(access: A) -> CruiserResult<Self> {
        Ok(Self(access.try_map_ref(|x: &[u8]| {
            if x.len() < 32 {
                Err(GenericError::NotEnoughData {
                    needed: 32,
                    remaining: x.len(),
                })
            } else {
                Ok((&x[..32]).try_into().unwrap())
            }
        })?))
    }
}

impl<'a, A> Deref for PubkeyAccess<'a, A>
where
    A: Deref<Target = [u8]> + TryMappableRef,
{
    type Target = Pubkey;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0.as_ptr().cast::<Pubkey>() }
    }
}

/// In-place accessor for [`Pubkey`]
#[derive(Debug)]
pub struct PubkeyAccessMut<'a, A>(A::Output<'a, [u8; 32]>)
where
    A: 'a + TryMappableRefMut;

impl<'a, A> PubkeyAccessMut<'a, A>
where
    A: DerefMut<Target = [u8]> + TryMappableRefMut,
{
    fn new(access: A) -> CruiserResult<Self> {
        Ok(Self(access.try_map_ref_mut(|x: &mut [u8]| {
            if x.len() < 32 {
                Err(GenericError::NotEnoughData {
                    needed: 32,
                    remaining: x.len(),
                })
            } else {
                Ok((&mut x[..32]).try_into().unwrap())
            }
        })?))
    }
}

impl<'a, A> Deref for PubkeyAccessMut<'a, A>
where
    A: DerefMut<Target = [u8]> + TryMappableRefMut,
{
    type Target = Pubkey;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0.as_ptr().cast::<Pubkey>() }
    }
}

impl<'a, A> DerefMut for PubkeyAccessMut<'a, A>
where
    A: DerefMut<Target = [u8]> + TryMappableRefMut,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
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
    fn read_with_arg<'a, A>(data: A, _arg: ()) -> CruiserResult<Self::Access<'a, A>>
    where
        Self: 'a,
        A: 'a + Deref<Target = [u8]> + MappableRef + TryMappableRef,
    {
        PubkeyAccess::new(data)
    }
}

impl InPlaceWrite for Pubkey {
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
        PubkeyAccessMut::new(data)
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
