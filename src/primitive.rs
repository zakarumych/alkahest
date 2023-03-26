use core::{borrow::Borrow, mem::size_of};

use crate::{
    cold::cold,
    deserialize::{Deserialize, DeserializeError, Deserializer},
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
            #[inline(always)]
            fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                let mut ser = ser.into();
                ser.write_bytes(&self.to_le_bytes())?;
                ser.finish()
            }

            #[inline(always)]
            fn size_hint(&self) -> Option<(usize, usize)> {
                Some((0, size_of::<$ty>()))
            }
        }

        impl Serialize<$ty> for &$ty {
            #[inline(always)]
            fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                let mut ser = ser.into();
                ser.write_bytes(&self.to_le_bytes())?;
                ser.finish()
            }

            #[inline(always)]
            fn size_hint(&self) -> Option<(usize, usize)> {
                Some((0, size_of::<$ty>()))
            }
        }

        impl<T> Deserialize<'_, $ty> for T
        where
            T: From<$ty>,
        {
            #[inline(always)]
            fn deserialize(de: Deserializer) -> Result<Self, DeserializeError> {
                let input = de.read_all_bytes();
                if input.len() == size_of::<$ty>() {
                    let mut bytes = [0; size_of::<$ty>()];
                    bytes.copy_from_slice(input);
                    let value = <$ty>::from_le_bytes(bytes);
                    return Ok(From::from(value));
                }

                cold();
                if input.len() > size_of::<$ty>() {
                    Err(DeserializeError::WrongLength)
                } else {
                    Err(DeserializeError::OutOfBounds)
                }
            }

            #[inline(always)]
            fn deserialize_in_place(&mut self, de: Deserializer) -> Result<(), DeserializeError> {
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
    #[inline(always)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        <u8 as Serialize<u8>>::serialize(*self.borrow() as u8, ser)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<(usize, usize)> {
        Some((0, size_of::<u8>()))
    }
}

impl Serialize<bool> for &bool {
    #[inline(always)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        <u8 as Serialize<u8>>::serialize(*self.borrow() as u8, ser)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<(usize, usize)> {
        Some((0, size_of::<u8>()))
    }
}

impl<T> Deserialize<'_, bool> for T
where
    T: From<bool>,
{
    #[inline(always)]
    fn deserialize(de: Deserializer) -> Result<Self, DeserializeError> {
        let value = <u8 as Deserialize<u8>>::deserialize(de)?;
        Ok(From::from(value != 0))
    }

    #[inline(always)]
    fn deserialize_in_place(&mut self, de: Deserializer) -> Result<(), DeserializeError> {
        let value = <u8 as Deserialize<u8>>::deserialize(de)?;
        *self = From::from(value != 0);
        Ok(())
    }
}
