use crate::{
    element::Element,
    list::List,
    serialize::{Serialize, Serializer, Sizes},
};

use alloc::vec::Vec;

impl<E, T> Serialize<List<E>> for Vec<T>
where
    E: Element,
    T: Serialize<E::Formula>,
{
    #[inline(always)]
    fn serialize<S>(&self, serializer: S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        Serialize::<List<E>>::serialize(&self[..], serializer)
    }

    #[inline(always)]
    fn size_hint<const SIZE_BYTES: u8>(&self) -> Option<Sizes> {
        Serialize::<List<E>>::size_hint::<SIZE_BYTES>(&self[..])
    }
}

formula_alias!(for[E: Element] Vec<E> as List<E>);
