use core::{borrow::Borrow, mem::size_of};

use crate::{
    cold::cold,
    deserialize::{Deserialize, Deserializer, Error},
    formula::{BareFormula, Formula},
    serialize::{Serialize, Serializer},
};

macro_rules! impl_primitive {
    () => {};

    ($head:ty $(, $tail:ty)* $(,)?) => {
        impl_primitive!(@ $head);
        impl_primitive!($($tail,)*);
    };

    (@ $ty:ty) => {
        impl Formula for $ty {
            const MAX_STACK_SIZE: Option<usize> = Some(size_of::<$ty>());
            const EXACT_SIZE: bool = true;
            const HEAPLESS: bool = true;
        }

        impl BareFormula for $ty {}

        impl Serialize<$ty> for $ty {
            #[inline(never)]
            fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                let mut ser = ser.into();
                ser.write_bytes(&self.to_le_bytes())?;
                ser.finish()
            }

            #[inline(never)]
            fn size_hint(&self) -> Option<usize> {
                Some(size_of::<$ty>())
            }
        }

        impl Serialize<$ty> for &$ty {
            #[inline(never)]
            fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                let mut ser = ser.into();
                ser.write_bytes(&self.to_le_bytes())?;
                ser.finish()
            }

            #[inline(never)]
            fn size_hint(&self) -> Option<usize> {
                Some(size_of::<$ty>())
            }
        }

        impl<T> Deserialize<'_, $ty> for T
        where
            T: From<$ty>,
        {
            #[inline(never)]
            fn deserialize(de: Deserializer) -> Result<Self, Error> {
                let input = de.read_all_bytes();
                if input.len() == size_of::<$ty>() {
                    let mut bytes = [0; size_of::<$ty>()];
                    bytes.copy_from_slice(input);
                    let value = <$ty>::from_le_bytes(bytes);
                    return Ok(From::from(value));
                }

                cold();
                if input.len() > size_of::<$ty>() {
                    Err(Error::WrongLength)
                } else {
                    Err(Error::OutOfBounds)
                }
            }

            #[inline(never)]
            fn deserialize_in_place(&mut self, de: Deserializer) -> Result<(), Error> {
                let value = <T as Deserialize<'_, $ty>>::deserialize(de)?;
                *self = value;
                Ok(())
            }
        }
    };
}

impl_primitive! {
    u8,
    u16,
    u32,
    u64,
    u128,
    i8,
    i16,
    i32,
    i64,
    i128,
    f32,
    f64,
}

impl Formula for bool {
    const MAX_STACK_SIZE: Option<usize> = Some(1);
    const EXACT_SIZE: bool = true;
    const HEAPLESS: bool = true;
}

impl BareFormula for bool {}

impl Serialize<bool> for bool {
    #[inline(never)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        <u8 as Serialize<u8>>::serialize(*self.borrow() as u8, ser)
    }

    #[inline(never)]
    fn size_hint(&self) -> Option<usize> {
        Some(size_of::<u8>())
    }
}

impl Serialize<bool> for &bool {
    #[inline(never)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        <u8 as Serialize<u8>>::serialize(*self.borrow() as u8, ser)
    }

    #[inline(never)]
    fn size_hint(&self) -> Option<usize> {
        Some(size_of::<u8>())
    }
}

impl<T> Deserialize<'_, bool> for T
where
    T: From<bool>,
{
    #[inline(never)]
    fn deserialize(de: Deserializer) -> Result<Self, Error> {
        let value = <u8 as Deserialize<u8>>::deserialize(de)?;
        Ok(From::from(value != 0))
    }

    #[inline(never)]
    fn deserialize_in_place(&mut self, de: Deserializer) -> Result<(), Error> {
        let value = <u8 as Deserialize<u8>>::deserialize(de)?;
        *self = From::from(value != 0);
        Ok(())
    }
}
