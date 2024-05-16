use core::mem::size_of;

use crate::{
    buffer::Buffer,
    deserialize::{Deserialize, DeserializeError, Deserializer},
    formula::{BareFormula, Formula},
    serialize::{write_bytes, Serialize, SerializeRef, Sizes},
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
            const MAX_STACK_SIZE: Option<usize> = Some(size_of::<$ty>());
            const EXACT_SIZE: bool = true;
            const HEAPLESS: bool = true;
        }

        impl BareFormula for $ty {}

        impl Serialize<$ty> for $ty {
            #[inline(always)]
            fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
            where
                B: Buffer,
            {
                write_bytes(&self.to_le_bytes(), sizes, buffer)
            }

            #[inline(always)]
            fn size_hint(&self) -> Option<Sizes> {
                Some(Sizes{ heap: 0, stack: size_of::<$ty>()})
            }
        }

        $(
            impl Serialize<$ty> for $from {
                #[inline(always)]
                fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
                where
                    B: Buffer,
                {
                    write_bytes(&$ty::from(self).to_le_bytes(), sizes, buffer)
                }

                #[inline(always)]
                fn size_hint(&self) -> Option<Sizes> {
                    Some(Sizes{ heap: 0, stack: size_of::<$ty>()})
                }
            }
        )*

        impl SerializeRef<$ty> for $ty {
            #[inline(always)]
            fn serialize<B>(&self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
            where
                B: Buffer,
            {
                write_bytes(&self.to_le_bytes(), sizes, buffer)
            }

            #[inline(always)]
            fn size_hint(&self) -> Option<Sizes> {
                Some(Sizes{ heap: 0, stack: size_of::<$ty>()})
            }
        }

        impl Deserialize<'_, $ty> for $ty
        {
            #[inline(always)]
            fn deserialize(mut de: Deserializer) -> Result<Self, DeserializeError> {
                let input = de.read_byte_array::<{size_of::<$ty>()}>()?;
                // de.finish()?;
                let value = <$ty>::from_le_bytes(input);
                return Ok(value);
            }

            #[inline(always)]
            fn deserialize_in_place(&mut self, mut de: Deserializer) -> Result<(), DeserializeError> {
                let input = de.read_byte_array::<{size_of::<$ty>()}>()?;
                // de.finish()?;
                let value = <$ty>::from_le_bytes(input);
                *self = value;
                Ok(())
            }
        }

        $(
            impl SerializeRef<$ty> for $from {
                #[inline(always)]
                fn serialize<B>(&self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
                where
                    B: Buffer,
                {
                    write_bytes(&$ty::from(*self).to_le_bytes(), sizes, buffer)
                }

                #[inline(always)]
                fn size_hint(&self) -> Option<Sizes> {
                    Some(Sizes{ heap: 0, stack: size_of::<$ty>()})
                }
            }

            impl Deserialize<'_, $from> for $ty
            {
                #[inline(always)]
                fn deserialize(mut de: Deserializer) -> Result<Self, DeserializeError> {
                    let input = de.read_byte_array::<{size_of::<$from>()}>()?;
                    // de.finish()?;
                    let value = <$from>::from_le_bytes(input);
                    return Ok($ty::from(value));
                }

                #[inline(always)]
                fn deserialize_in_place(&mut self, mut de: Deserializer) -> Result<(), DeserializeError> {
                    let input = de.read_byte_array::<{size_of::<$from>()}>()?;
                    // de.finish()?;
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
    const MAX_STACK_SIZE: Option<usize> = Some(1);
    const EXACT_SIZE: bool = true;
    const HEAPLESS: bool = true;
}

impl BareFormula for bool {}

impl Serialize<bool> for bool {
    #[inline(always)]
    fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        Self: Sized,
        B: Buffer,
    {
        write_bytes(&[u8::from(self)], sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        Some(Sizes {
            heap: 0,
            stack: size_of::<u8>(),
        })
    }
}

impl Serialize<bool> for &bool {
    #[inline(always)]
    fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        <u8 as Serialize<u8>>::serialize(u8::from(*self), sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        Some(Sizes {
            heap: 0,
            stack: size_of::<u8>(),
        })
    }
}

impl Deserialize<'_, bool> for bool {
    #[inline(always)]
    fn deserialize(mut de: Deserializer) -> Result<Self, DeserializeError> {
        let byte = de.read_byte()?;
        Ok(byte != 0)
    }

    #[inline(always)]
    fn deserialize_in_place(&mut self, mut de: Deserializer) -> Result<(), DeserializeError> {
        let byte = de.read_byte()?;
        *self = byte != 0;
        Ok(())
    }
}
