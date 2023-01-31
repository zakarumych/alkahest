use core::{borrow::Borrow, mem::size_of};

use crate::{
    deserialize::{Deserialize, DeserializeError},
    schema::{Schema, SizedSchema},
    serialize::Serialize,
};

macro_rules! impl_primitive {
    () => {};

    ($head:ty $(, $tail:ty)* $(,)?) => {
        impl_primitive!(impl $head);
        impl_primitive!($($tail,)*);
    };

    (impl $ty:ty) => {
        impl Schema for $ty {}
        impl SizedSchema for $ty {
            const SIZE: usize = size_of::<$ty>();
        }

        impl<T> Serialize<$ty> for T
        where
            T: Borrow<$ty>,
        {
            #[inline(always)]
            fn serialize(self, _offset: usize, output: &mut [u8]) -> Result<(usize, usize), usize> {
                const SIZE: usize = size_of::<$ty>();

                if output.len() < SIZE {
                    return Err(SIZE);
                };

                let bytes = self.borrow().to_le_bytes();
                let at = output.len() - SIZE;
                output[at..].copy_from_slice(&bytes);
                Ok((0, at))
            }
        }

        impl<T> Deserialize<'_, $ty> for T
        where
            T: From<$ty>,
        {
            #[inline(always)]
            fn deserialize(len: usize, input: &[u8]) -> Result<Self, DeserializeError> {
                const SIZE: usize = size_of::<$ty>();

                if len != SIZE {
                    return Err(DeserializeError::WrongLength);
                }

                if input.len() < SIZE {
                    return Err(DeserializeError::OutOfBounds);
                }

                let mut bytes = [0; SIZE];
                let at = input.len() - SIZE;
                bytes.copy_from_slice(&input[at..]);

                let value = <$ty>::from_le_bytes(bytes);
                Ok(From::from(value))
            }

            #[inline(always)]
            fn deserialize_in_place(&mut self, len: usize, input: &[u8]) -> Result<(), DeserializeError> {
                let value = <T as Deserialize<'_, $ty>>::deserialize(len, input)?;
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

impl Schema for bool {}
impl SizedSchema for bool {
    const SIZE: usize = 1;
}

impl<T> Serialize<bool> for T
where
    T: Borrow<bool>,
{
    #[inline(always)]
    fn serialize(self, offset: usize, output: &mut [u8]) -> Result<(usize, usize), usize> {
        <u8 as Serialize<u8>>::serialize((*self.borrow()) as u8, offset, output)
    }
}

impl<T> Deserialize<'_, bool> for T
where
    T: From<bool>,
{
    #[inline(always)]
    fn deserialize(len: usize, input: &[u8]) -> Result<Self, DeserializeError> {
        let value = <u8 as Deserialize<u8>>::deserialize(len, input)?;
        Ok(From::from(value != 0))
    }

    #[inline(always)]
    fn deserialize_in_place(&mut self, len: usize, input: &[u8]) -> Result<(), DeserializeError> {
        let value = <u8 as Deserialize<u8>>::deserialize(len, input)?;
        *self = From::from(value != 0);
        Ok(())
    }
}
