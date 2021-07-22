use {
    crate::schema::{OwnedSchema, Pack, Schema, SchemaUnpack},
    core::mem::align_of,
};

macro_rules! impl_primitive {
    ($head:ty $(, $tail:ty)+ $(,)?) => {
        impl_primitive!($head);
        impl_primitive!($($tail),+);
    };
    ($ty:ty) => {
        impl<'a> SchemaUnpack<'a> for $ty {
            type Unpacked = Self;
        }

        impl Schema for $ty {
            type Packed = Self;

            #[inline(always)]
            fn align() -> usize {
                align_of::<$ty>()
            }

            #[inline(always)]
            #[cfg(target_endian = "little")]
            fn unpack<'a>(packed: $ty, _bytes: &'a [u8]) -> $ty {
                packed
            }

            #[inline(always)]
            #[cfg(not(target_endian = "little"))]
            fn unpack<'a>(packed: $ty, _bytes: &'a [u8]) -> $ty {
                <$ty>::from_le(packed)
            }
        }

        impl<T> Pack<$ty> for T
        where
            T: core::borrow::Borrow<$ty>,
        {
            #[inline(always)]
            #[cfg(target_endian = "little")]
            fn pack(self, _offset: usize, _bytes: &mut [u8]) -> ($ty, usize) {
                (*self.borrow(), 0)
            }

            #[inline(always)]
            #[cfg(not(target_endian = "little"))]
            fn pack(self, _offset: usize, _bytes: &mut [u8]) -> ($ty, usize) {
                (<$ty>::to_le(*self.borrow()), 0)
            }
        }

        impl OwnedSchema for $ty {
            #[inline(always)]
            fn to_owned(unpacked: $ty) -> $ty {
                unpacked
            }
        }
    };
}

impl_primitive!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64);

impl<'a> SchemaUnpack<'a> for bool {
    type Unpacked = bool;
}

impl Schema for bool {
    type Packed = u8;

    #[inline(always)]
    fn align() -> usize {
        align_of::<u8>()
    }

    #[inline(always)]
    fn unpack<'a>(packed: u8, _bytes: &'a [u8]) -> bool {
        packed != 0
    }
}

impl<T> Pack<bool> for T
where
    T: core::borrow::Borrow<bool>,
{
    #[inline(always)]
    fn pack(self, _offset: usize, _bytes: &mut [u8]) -> (u8, usize) {
        (*self.borrow() as u8, 0)
    }
}

impl OwnedSchema for bool {
    fn to_owned<'a>(unpacked: bool) -> bool {
        unpacked
    }
}
