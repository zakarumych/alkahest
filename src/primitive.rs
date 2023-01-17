use core::mem::size_of;

use crate::schema::{Access, Schema, Serialize};

macro_rules! impl_primitive {
    ($head:ty $(, $tail:ty)+ $(,)?) => {
        impl_primitive!($head);
        impl_primitive!($($tail),+);
    };
    ($ty:ty) => {
        impl Schema for $ty {
            type Access<'a> = Self;

            #[inline(always)]
            fn header() -> usize {
                size_of::<$ty>()
            }

            #[inline(always)]
            fn has_body() -> bool {
                false
            }

            #[inline(always)]
            fn access<'a>(input: &'a [u8]) -> $ty {
                if input.len() < size_of::<$ty>() {
                    cold_panic!("input buffer is too small");
                }
                let array: [_; size_of::<$ty>()] = input[..size_of::<$ty>()].try_into().unwrap();
                <$ty>::from_le_bytes(array)
            }
        }

        impl<T> Serialize<$ty> for T
        where
            T: core::borrow::Borrow<$ty>,
        {
            type Header = $ty;

            #[inline(always)]
            fn serialize_body(self, _output: &mut [u8]) -> Result<($ty, usize), usize> {
                Ok((*self.borrow(), 0))
            }

            #[inline(always)]
            fn serialize_header(header: $ty, output: &mut [u8], _offset: usize) -> bool {
                if output.len() < size_of::<$ty>() {
                    return false;
                }
                let array: &mut [_; size_of::<$ty>()] = (&mut output[..size_of::<$ty>()]).try_into().unwrap();
                *array = header.to_le_bytes();
                true
            }
        }
    };
}

impl_primitive!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64);

impl Schema for bool {
    type Access<'a> = Self;

    #[inline(always)]
    fn header() -> usize {
        <u8 as Schema>::header()
    }

    #[inline(always)]
    fn has_body() -> bool {
        <u8 as Schema>::has_body()
    }

    #[inline(always)]
    fn access<'a>(input: &'a [u8]) -> Access<'a, Self> {
        <u8 as Schema>::access(input) != 0
    }
}

impl<T> Serialize<bool> for T
where
    T: core::borrow::Borrow<bool>,
{
    type Header = u8;

    #[inline(always)]
    fn serialize_body(self, output: &mut [u8]) -> Result<(u8, usize), usize> {
        let v = *self.borrow() as u8;
        <u8 as Serialize<u8>>::serialize_body(v, output)
    }

    #[inline(always)]
    fn serialize_header(header: u8, output: &mut [u8], offset: usize) -> bool {
        <u8 as Serialize<u8>>::serialize_header(header, output, offset)
    }
}
