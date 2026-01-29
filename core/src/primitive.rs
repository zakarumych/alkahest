use core::mem::size_of;

use crate::{
    deserialize::{Deserialize, DeserializeError, Deserializer},
    formula::Formula,
    serialize::{Serialize, Serializer, Sizes},
};

macro_rules! impl_primitive {
    () => {};

    ([$($head:ident)+] $([$($tail:ident)+])*) => {
        impl_primitive!(^ < $($head)+);
        impl_primitive!($([$($tail)+])*);
    };

    (^ $($head:ident)* <) => {};
    (^ $($head:ident)* < $cursor:ident $($tail:ident)*) => {
        impl_primitive!{! $($head)* < $cursor < $($tail)* }
        impl_primitive!{^ $($head)* $cursor < $($tail)* }
    };

    (! $($from:ident)* < $ty:ident < $($to:ident)*) => {
        impl<const SIZE_BYTES: u8> Formula<SIZE_BYTES> for $ty {
            const MAX_STACK_SIZE: Option<usize> = Some(size_of::<$ty>());
            const EXACT_SIZE: bool = true;
            const HEAPLESS: bool = true;
        }

        impl<const SIZE_BYTES: u8> Serialize<$ty, SIZE_BYTES> for $ty {
            #[inline(always)]
            fn serialize<S>(&self, mut serializer: S) -> Result<(), S::Error>
            where
                S: Serializer<SIZE_BYTES>,
            {
                serializer.write_bytes(&self.to_le_bytes())
            }

            #[inline(always)]
            fn size_hint(&self) -> Option<Sizes> {
                Some(Sizes{ heap: 0, stack: size_of::<$ty>()})
            }
        }

        $(
            impl<const SIZE_BYTES: u8> Serialize<$ty, SIZE_BYTES> for $from {
                #[inline(always)]
                fn serialize<S>(&self, mut serializer: S) -> Result<(), S::Error>
                where
                    S: Serializer<SIZE_BYTES>,
                {
                    serializer.write_bytes(&$ty::from(*self).to_le_bytes())
                }

                #[inline(always)]
                fn size_hint(&self) -> Option<Sizes> {
                    Some(Sizes{ heap: 0, stack: size_of::<$ty>()})
                }
            }
        )*


        impl<'de, const SIZE_BYTES: u8> Deserialize<'de, $ty, SIZE_BYTES> for $ty
        {
            #[inline(always)]
            fn deserialize<D>(mut de: D) -> Result<Self, DeserializeError>
            where
                D: Deserializer<'de, SIZE_BYTES>,
             {
                let input = de.read_byte_array::<{size_of::<$ty>()}>()?;
                // de.finish()?;
                let value = <$ty>::from_le_bytes(input);
                return Ok(value);
            }

            #[inline(always)]
            fn deserialize_in_place<D>(&mut self, mut de: D) -> Result<(), DeserializeError>
            where
                D: Deserializer<'de, SIZE_BYTES>,
            {
                let input = de.read_byte_array::<{size_of::<$ty>()}>()?;
                // de.finish()?;
                let value = <$ty>::from_le_bytes(input);
                *self = value;
                Ok(())
            }
        }

        $(
            impl<'de, const SIZE_BYTES: u8> Deserialize<'de, $from, SIZE_BYTES> for $ty
            {
                #[inline(always)]
                fn deserialize<D>(mut de: D) -> Result<Self, DeserializeError>
                where
                    D: Deserializer<'de, SIZE_BYTES>,
                {
                    let input = de.read_byte_array::<{size_of::<$from>()}>()?;
                    let value = <$from>::from_le_bytes(input);
                    return Ok($ty::from(value));
                }

                #[inline(always)]
                fn deserialize_in_place<D>(&mut self, mut de: D) -> Result<(), DeserializeError>
                where
                    D: Deserializer<'de, SIZE_BYTES>,
                {
                    let input = de.read_byte_array::<{size_of::<$from>()}>()?;
                    let value = <$from>::from_le_bytes(input);
                    *self = $ty::from(value);
                    Ok(())
                }
            }
        )*
    };
}

impl_primitive! {
    [u8 u16 u32 u64 u128]
    [i8 i16 i32 i64 i128]
    [f32 f64]
}

impl<const SIZE_BYTES: u8> Formula<SIZE_BYTES> for bool {
    const MAX_STACK_SIZE: Option<usize> = Some(1);
    const EXACT_SIZE: bool = true;
    const HEAPLESS: bool = true;
}

impl<const SIZE_BYTES: u8> Serialize<bool, SIZE_BYTES> for bool {
    #[inline(always)]
    fn serialize<S>(&self, mut serializer: S) -> Result<(), S::Error>
    where
        Self: Sized,
        S: Serializer<SIZE_BYTES>,
    {
        serializer.write_bytes(&[u8::from(*self)])
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        Some(Sizes {
            heap: 0,
            stack: size_of::<u8>(),
        })
    }
}

impl<'de, const SIZE_BYTES: u8> Deserialize<'de, bool, SIZE_BYTES> for bool {
    #[inline(always)]
    fn deserialize<D>(mut de: D) -> Result<Self, DeserializeError>
    where
        D: Deserializer<'de, SIZE_BYTES>,
    {
        let byte = de.read_byte()?;
        Ok(byte != 0)
    }

    #[inline(always)]
    fn deserialize_in_place<D>(&mut self, mut de: D) -> Result<(), DeserializeError>
    where
        D: Deserializer<'de, SIZE_BYTES>,
    {
        let byte = de.read_byte()?;
        *self = byte != 0;
        Ok(())
    }
}
