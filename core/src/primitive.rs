use core::mem::size_of;

use crate::{
    deserialize::{Deserialize, DeserializeError, Deserializer},
    formula::{ExactSize, Formula},
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
        impl Formula for $ty {
            type StackSize<const SIZE_BYTES: u8> = ExactSize<{size_of::<$ty>()}>;
            type HeapSize<const SIZE_BYTES: u8> = ExactSize<0>;
            const INHABITED: bool = true;
        }

        impl Serialize<$ty> for $ty {
            #[inline]
            fn serialize<S>(&self, mut serializer: S) -> Result<(), S::Error>
            where
                S: Serializer,
            {
                serializer.write_bytes(&self.to_le_bytes())
            }

            #[inline]
            fn size_hint<const SIZE_BYTES: u8>(&self) -> Option<Sizes> {
                Some(Sizes{ heap: 0, stack: size_of::<$ty>()})
            }
        }

        $(
            impl Serialize<$ty> for $from {
                #[inline]
                fn serialize<S>(&self, mut serializer: S) -> Result<(), S::Error>
                where
                    S: Serializer,
                {
                    serializer.write_bytes(&$ty::from(*self).to_le_bytes())
                }

                #[inline]
                fn size_hint<const SIZE_BYTES: u8>(&self) -> Option<Sizes> {
                    Some(Sizes{ heap: 0, stack: size_of::<$ty>()})
                }
            }
        )*


        impl<'de> Deserialize<'de, $ty> for $ty
        {
            #[inline]
            fn deserialize<D>(mut de: D) -> Result<Self, DeserializeError>
            where
                D: Deserializer<'de>,
             {
                let input = de.read_byte_array::<{size_of::<$ty>()}>()?;
                // de.finish()?;
                let value = <$ty>::from_le_bytes(input);
                return Ok(value);
            }

            #[inline]
            fn deserialize_in_place<D>(&mut self, mut de: D) -> Result<(), DeserializeError>
            where
                D: Deserializer<'de>,
            {
                let input = de.read_byte_array::<{size_of::<$ty>()}>()?;
                // de.finish()?;
                let value = <$ty>::from_le_bytes(input);
                *self = value;
                Ok(())
            }
        }

        $(
            impl<'de> Deserialize<'de, $from> for $ty
            {
                #[inline]
                fn deserialize<D>(mut de: D) -> Result<Self, DeserializeError>
                where
                    D: Deserializer<'de>,
                {
                    let input = de.read_byte_array::<{size_of::<$from>()}>()?;
                    let value = <$from>::from_le_bytes(input);
                    return Ok($ty::from(value));
                }

                #[inline]
                fn deserialize_in_place<D>(&mut self, mut de: D) -> Result<(), DeserializeError>
                where
                    D: Deserializer<'de>,
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

impl Formula for bool {
    type StackSize<const SIZE_BYTES: u8> = ExactSize<1>;
    type HeapSize<const SIZE_BYTES: u8> = ExactSize<0>;
    const INHABITED: bool = true;
}

impl Serialize<bool> for bool {
    #[inline]
    fn serialize<S>(&self, mut serializer: S) -> Result<(), S::Error>
    where
        Self: Sized,
        S: Serializer,
    {
        serializer.write_bytes(&[u8::from(*self)])
    }

    #[inline]
    fn size_hint<const SIZE_BYTES: u8>(&self) -> Option<Sizes> {
        Some(Sizes {
            heap: 0,
            stack: size_of::<u8>(),
        })
    }
}

impl<'de> Deserialize<'de, bool> for bool {
    #[inline]
    fn deserialize<D>(mut de: D) -> Result<Self, DeserializeError>
    where
        D: Deserializer<'de>,
    {
        let byte = de.read_byte()?;
        Ok(byte != 0)
    }

    #[inline]
    fn deserialize_in_place<D>(&mut self, mut de: D) -> Result<(), DeserializeError>
    where
        D: Deserializer<'de>,
    {
        let byte = de.read_byte()?;
        *self = byte != 0;
        Ok(())
    }
}
