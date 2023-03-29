use core::{borrow::Borrow, mem::size_of};

use crate::{
    buffer::Buffer,
    deserialize::{Deserialize, DeserializeError, Deserializer},
    formula::{BareFormula, Formula},
    serialize::{write_bytes, Serialize, Sizes},
};

macro_rules! impl_primitive {
    () => {};

    ([$($head:ident)+] $([$($tail:ident)+])*) => {
        impl_primitive!(@ < $($head)+);
        impl_primitive!($([$($tail)+])*);
    };

    (@ $($head:ident)* <) => {};
    (@ $($head:ident)* < $cursor:ident $($tail:ident)*) => {
        impl_primitive!{! $($head)* < $cursor < $($tail)* }
        impl_primitive!{@ $($head)* $cursor < $($tail)* }
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

        impl Serialize<$ty> for &$ty {
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
            impl Serialize<$ty> for &$from {
                #[inline(always)]
                fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
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
        )*

        impl<T> Deserialize<'_, $ty> for T
        where
            T: From<$ty>,
        {
            #[inline(always)]
            fn deserialize(mut de: Deserializer) -> Result<Self, DeserializeError> {
                let input = de.read_bytes(size_of::<$ty>())?;
                de.finish()?;
                let mut bytes = [0; size_of::<$ty>()];
                bytes.copy_from_slice(input);
                let value = <$ty>::from_le_bytes(bytes);
                return Ok(From::from(value));
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
        write_bytes(&[self as u8], sizes, buffer)
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
        <u8 as Serialize<u8>>::serialize(*self.borrow() as u8, sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        Some(Sizes {
            heap: 0,
            stack: size_of::<u8>(),
        })
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
