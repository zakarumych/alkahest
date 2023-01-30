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

                let bytes = <T as Borrow<$ty>>::borrow(&self).to_le_bytes();
                let at = output.len() - SIZE;
                output[at..].copy_from_slice(&bytes);
                Ok((SIZE, 0))
            }
        }

        impl<T> Deserialize<$ty> for T
        where
            T: From<$ty>
        {
            fn deserialize(input: &[u8]) -> Result<Self, DeserializeError> {
                const SIZE: usize = size_of::<$ty>();

                if input.len() < SIZE {
                    return Err(DeserializeError::OutOfBounds);
                }

                let mut bytes = [0; SIZE];
                let at = input.len() - SIZE;
                bytes.copy_from_slice(&input[at..]);

                let value = <$ty>::from_le_bytes(bytes);
                Ok(From::from(value))
            }

            fn deserialize_in_place(&mut self, input: &[u8]) -> Result<(), DeserializeError> {
                *self = <Self as Deserialize<$ty>>::deserialize(input)?;
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

impl<T> Serialize<bool> for T
where
    T: Borrow<bool>,
{
    #[inline(always)]
    fn serialize(self, offset: usize, output: &mut [u8]) -> Result<(usize, usize), usize> {
        <u8 as Serialize<u8>>::serialize((*self.borrow()) as u8, offset, output)
    }
}

impl<T> Deserialize<bool> for T
where
    T: From<bool>,
{
    #[inline(always)]
    fn deserialize(input: &[u8]) -> Result<Self, DeserializeError> {
        let value = <u8 as Deserialize<u8>>::deserialize(input)? != 0;
        Ok(From::from(value))
    }

    #[inline(always)]
    fn deserialize_in_place(&mut self, input: &[u8]) -> Result<(), DeserializeError> {
        *self = <Self as Deserialize<bool>>::deserialize(input)?;
        Ok(())
    }
}
