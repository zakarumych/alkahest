use core::{borrow::Borrow, mem::size_of};

use crate::{
    deserialize::{Deserialize, DeserializeError},
    schema::Schema,
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
            fn deserialize(input: &[u8]) -> Result<(Self, usize), DeserializeError> {
                const SIZE: usize = size_of::<$ty>();

                if input.len() < SIZE {
                    return Err(DeserializeError::OutOfBounds);
                }

                let mut bytes = [0; SIZE];
                let at = input.len() - SIZE;
                bytes.copy_from_slice(&input[at..]);

                let value = <$ty>::from_le_bytes(bytes);
                Ok((From::from(value), SIZE))
            }

            #[inline(always)]
            fn deserialize_in_place(&mut self, input: &[u8]) -> Result<usize, DeserializeError> {
                let (value, size) = <T as Deserialize<'_, $ty>>::deserialize(input)?;
                *self = value;
                Ok(size)
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
    fn deserialize(input: &[u8]) -> Result<(Self, usize), DeserializeError> {
        let (value, size) = <u8 as Deserialize<u8>>::deserialize(input)?;
        Ok((From::from(value != 0), size))
    }

    #[inline(always)]
    fn deserialize_in_place(&mut self, input: &[u8]) -> Result<usize, DeserializeError> {
        let (value, size) = <u8 as Deserialize<u8>>::deserialize(input)?;
        *self = From::from(value != 0);
        Ok(size)
    }
}
